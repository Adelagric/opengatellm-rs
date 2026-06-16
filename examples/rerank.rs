//! Exemple : reranking d'une liste de documents par pertinence.
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 \
//! OGL_TOKEN=mytoken \
//! OGL_RERANK_MODEL=bge-reranker-v2-m3 \
//! cargo run --example rerank
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use opengatellm::{Client, CreateRerankBody};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();
    let model = std::env::var("OGL_RERANK_MODEL").unwrap_or_else(|_| "bge-reranker-v2-m3".into());

    let client = Client::new(url, token)?;

    let query = "aides à l'investissement pour l'irrigation";
    let documents = vec![
        "Prêt à taux zéro pour la trésorerie agricole.".to_owned(),
        "Subvention pour le matériel d'irrigation économe en eau.".to_owned(),
        "Aide à la certification agriculture biologique.".to_owned(),
    ];

    let req = CreateRerankBody::new(query, documents.clone(), model);
    let resp = client.rerank(&req).await?;

    println!("query : {query}");
    for r in &resp.results {
        let doc = usize::try_from(r.index)
            .ok()
            .and_then(|i| documents.get(i))
            .map_or("?", String::as_str);
        println!("  score={:.4}  [{}] {}", r.relevance_score, r.index, doc);
    }
    Ok(())
}
