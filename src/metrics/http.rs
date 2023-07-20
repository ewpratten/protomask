use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use prometheus::{Encoder, TextEncoder};

/// Handle an HTTP request
async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    // If the request is targeting the metrics endpoint
    if request.method() == Method::GET && request.uri().path() == "/metrics" {
        // Gather metrics
        let metric_families = prometheus::gather();
        let body = {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        };

        // Return the response
        return Ok(Response::new(Body::from(body)));
    }

    // Otherwise, just return a 404
    Ok(Response::builder()
        .status(404)
        .body(Body::from("Not found"))
        .unwrap())
}

/// Bring up an HTTP server that listens for metrics requests
pub async fn serve_metrics(bind_addr: SocketAddr) {
    // Set up the server
    let make_service =
        make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let server = Server::bind(&bind_addr).serve(make_service);

    // Run the server
    if let Err(e) = server.await {
        eprintln!("Metrics server error: {}", e);
    }
}
