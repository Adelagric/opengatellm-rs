//! Exemple : santé du gateway et suivi de sa propre consommation.
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 OGL_TOKEN=mytoken cargo run --example monitoring
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use opengatellm::{Client, UsageQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();

    let client = Client::new(url, token)?;

    // Liveness du gateway (non authentifié côté OGL).
    println!("santé gateway : {}", client.health().await?.status);

    // Santé par modèle.
    match client.health_models().await {
        Ok(h) => {
            for m in &h.data {
                println!("  {} : {:?}", m.id, m.status);
            }
        }
        Err(e) => println!("health/models indisponible : {e}"),
    }

    // Sa propre consommation (10 dernières lignes).
    match client.usage(&UsageQuery::new().limit(10)).await {
        Ok(u) => println!("{} lignes d'usage", u.data.len()),
        Err(e) => println!("usage indisponible : {e}"),
    }
    Ok(())
}
