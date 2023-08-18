use autometrics::{autometrics, prometheus_exporter::encode_to_string};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rand::{prelude::*, random, thread_rng};
use std::{net::SocketAddr, time::Duration};

// Starting simple, hover over the function name to see the Autometrics graph links in the Rust Docs!
/// This is a simple endpoint that never errors
#[autometrics]
pub async fn get_index() -> &'static str {
    "Hello, World!"
}

#[autometrics]
pub async fn post_slow() -> Result<(), ()> {
    let sleep_duration = thread_rng().gen_range(3u8..10u8);
    tokio::time::sleep(Duration::from_secs(sleep_duration.into())).await;
    Ok(())
}

#[autometrics]
pub async fn post_random_error() -> Result<(), ()> {
    let should_error = random::<u8>();

    if should_error > 172u8 {
        Err(())
    } else {
        Ok(())
    }
}

/// This function doesn't return a Result, but we can determine whether
/// we want to consider it a success or not by passing a function to the `ok_if` parameter.
#[autometrics(ok_if = is_success)]
pub async fn route_that_returns_into_response() -> impl IntoResponse {
    (StatusCode::OK, "Hello, World!")
}

/// Determine whether the response was a success or not
fn is_success<R>(response: &R) -> bool
where
    R: Copy + IntoResponse,
{
    response.into_response().status().is_success()
}

/// This handler serializes the metrics into a string for Prometheus to scrape
pub async fn get_metrics() -> (StatusCode, String) {
    match encode_to_string() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)),
    }
}

#[tokio::main]
pub async fn main() {
    let app = Router::new()
        .route("/", get(get_index))
        .route("/random-error", post(post_random_error))
        .route("/slow", post(post_slow))
        // Expose the metrics for Prometheus to scrape
        .route("/metrics", get(get_metrics));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum::Server::bind(&addr);

    println!(
        "The example API server is now running on: {addr}

Wait a few seconds for the traffic generator to create some fake traffic.
Then, hover over one of the HTTP handler functions (in your editor) to bring up the Rust Docs.

Click on one of the Autometrics links to see the graph for that handler's metrics in Prometheus."
    );

    server
        .serve(app.into_make_service())
        .await
        .expect("Error starting example API server");
}
