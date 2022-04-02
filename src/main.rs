use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use rattice::{auth, config::Config, handle, trace};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = cli::Opt::init()?;
    if let Some(path) = &opt.docroot {
        tracing::info!("set document root to {}", path.display());
        std::env::set_current_dir(path)?;
    }

    let mut app = handle::add_handler(Router::new());
    if opt.username.is_some() || opt.password.is_some() {
        tracing::info!("Basic Authentication enabled");
        app = auth::add_basic_authentication(app, &opt.username, &opt.password);
    }

    app = app.layer(Extension(Arc::new(Config::new(
        !opt.eager,
        opt.title_prefix.clone(),
    ))));

    let app = trace::add_trace_layer(app, opt.use_real_ip, opt.verbose);
    let addr = format!("{}:{}", opt.bind_address, opt.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("Address is invalid {}:{}", opt.bind_address, opt.port))?;

    match (&opt.server_cert, &opt.server_key) {
        (Some(cert), Some(key)) => {
            tracing::info!("HTTPS enabled");
            let config = RustlsConfig::from_pem_file(cert, key).await?;

            tracing::info!("listening on {}", addr);
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        }
        _ => {
            tracing::info!("listening on {}", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        }
    }

    Ok(())
}
