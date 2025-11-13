use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use rattice::{auth, config::Config, generate, handle, trace};

mod cli;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = cli::Opt::init()?;
    if let Some(path) = &opt.docroot {
        tracing::info!("set document root to {}", path.display());
        std::env::set_current_dir(path)?;
    }

    let config = Arc::new(Config::new(
        !opt.eager,
        opt.title_prefix.clone(),
        opt.sort_order()?,
        opt.reverse,
        opt.depth,
        opt.ignore_query_params,
        opt.filter_dir.clone(),
        opt.filter_file.clone(),
    ));

    if opt.generate_static_pages {
        return generate::generate_static_pages(config);
    }

    let mut app = handle::add_handler(Router::new());
    if opt.username.is_some() || opt.password.is_some() {
        tracing::info!("Basic Authentication enabled");
        app = auth::add_basic_authentication(app, &opt.username, &opt.password);
    }

    app = app.layer(Extension(config));

    let app = trace::add_trace_layer(app, opt.real_ip_header.clone(), opt.verbose);
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
