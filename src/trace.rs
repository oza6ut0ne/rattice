use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing::Span;

pub fn add_trace_layer(app: Router, verbosity: u8) -> Router {
    app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let addr = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ci| ci.0.to_string())
                    .unwrap_or_else(|| "None".to_owned());
                let encoded_uri = request.uri().path().to_string();
                let decoded_uri =
                    percent_encoding::percent_decode_str(&encoded_uri).decode_utf8_lossy();
                tracing::info_span!("", "{} {} {}", addr, request.method(), decoded_uri)
            })
            .on_request(move |request: &Request<_>, span: &Span| {
                let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                tracing::debug!(?id, "started processing request");

                let authorization = request
                    .headers()
                    .get("authorization")
                    .map(|a| a.to_str().unwrap_or(""))
                    .map(|a| a.strip_prefix("Basic ").unwrap_or(""))
                    .map(|a| base64::decode(a).unwrap_or_default())
                    .map(|a| String::from_utf8_lossy(&a).to_string());

                if verbosity < 3 {
                    tracing::trace!(?id, "{:?}", request)
                } else if verbosity < 4 {
                    tracing::trace!(?authorization, ?id, "{:?}", request)
                } else {
                    tracing::trace!(?authorization, ?id, "{:#?}", request)
                }
            })
            .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                let id: i128 = span.id().map(|i| i.into_u64().into()).unwrap_or(-1);
                tracing::trace!(?id, "{:?}", response);
                tracing::info!(status = response.status().as_u16(), ?latency, ?id)
            }),
    )
}
