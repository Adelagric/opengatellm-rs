//! Exemple : embeddings d'un batch de textes.
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 \
//! OGL_TOKEN=mytoken \
//! OGL_EMBED_MODEL=nomic-embed-text \
//! cargo run --example embeddings
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use opengatellm::{Client, EmbeddingsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();
    let model = std::env::var("OGL_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".into());

    let client = Client::new(url, token)?;

    let inputs = vec![
        "Paris est la capitale de la France.".to_owned(),
        "Lyon se situe dans la région Auvergne-Rhône-Alpes.".to_owned(),
        "Marseille est un port méditerranéen.".to_owned(),
    ];

    let req = EmbeddingsRequest::new(inputs.clone(), model);
    let resp = client.embeddings(&req).await?;

    for (i, e) in resp.data.iter().enumerate() {
        println!(
            "[{:2}] dim={:4}  input={}",
            e.index,
            e.embedding.len(),
            &inputs[i]
        );
    }
    if let Some(usage) = &resp.usage {
        println!(
            "Usage : prompt_tokens={} total_tokens={}",
            usage.prompt_tokens, usage.total_tokens
        );
    }
    Ok(())
}
