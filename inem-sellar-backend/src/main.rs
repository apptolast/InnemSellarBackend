mod config;
mod db;

use salvo::prelude::*;

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new().get(hello);
    let acceptor = TcpListener::new("0.0.0.0:8080").bind().await;

    tracing::info!("Servidor escuchando en http://localhost:8080");
    Server::new(acceptor).serve(router).await;
}
