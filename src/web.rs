use std::{error::Error, net::SocketAddr};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(address).await?;
    let public_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/public");
    let app = Router::new()
        .fallback_service(ServeDir::new(public_dir).append_index_html_on_directories(true));

    println!("Serveur TOPRS disponible sur http://{address}");
    println!("Fichiers statiques servis depuis {public_dir}");

    axum::serve(listener, app).await?;
    Ok(())
}
