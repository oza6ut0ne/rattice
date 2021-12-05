use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{anyhow, Result};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rattice::{auth, handle, trace};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = cli::Opt::init();
    if let Some(path) = &opt.docroot {
        tracing::info!("set document root to {}", path.display());
        std::env::set_current_dir(path)?;
    }

    let mut app = handle::add_handler(Router::new());
    if opt.username.is_some() || opt.password.is_some() {
        tracing::info!("Basic Authentication enabled");
        app = auth::add_basic_authentication(app, &opt.username, &opt.password);
    }

    let app = trace::add_trace_layer(app, opt.verbose);
    let addr = format!("{}:{}", opt.bind_address, opt.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("Address is invalid {}:{}", opt.bind_address, opt.port))?;

    if opt.server_cert.is_some() && opt.server_key.is_some() {
        tracing::info!("HTTPS enabled");
        let config = RustlsConfig::from_pem_file(
            opt.server_cert.as_ref().unwrap(),
            opt.server_key.as_ref().unwrap(),
        )
        .await?;
        tracing::info!("listening on {}", addr);
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
            .await?;
    } else {
        tracing::info!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
            .await?;
    }

    Ok(())
}
