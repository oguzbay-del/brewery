//! Fake OIDC provider binary for the compose stack: `PORT` (default 9000) and `PUBLIC_URL`
//! (the address the API reaches it at, default `http://fake-oidc:9000`).

use std::net::SocketAddr;

use fake_oidc::FakeOidc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9000);
    let public_url =
        std::env::var("PUBLIC_URL").unwrap_or_else(|_| format!("http://fake-oidc:{port}"));
    let oidc = tokio::task::spawn_blocking(move || FakeOidc::new(&public_url)).await?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "fake-oidc listening");
    axum::serve(listener, oidc.router()).await?;
    Ok(())
}
