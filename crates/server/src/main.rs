use axum::Router;
use axum::routing::{get, post};
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/commit", post(create_user));

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080),
    ));
    println!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind to port");
    axum::serve(listener, app).await.expect("Failed to start server");
}
