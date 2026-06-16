//! Exemple : pipeline RAG minimal (collection → ingestion → recherche).
//!
//! Lancer :
//! ```sh
//! OGL_URL=http://localhost:8000 OGL_TOKEN=mytoken cargo run --example rag
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use opengatellm::{Client, CollectionRequest, CreateDocument, CreateSearch};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("OGL_URL").unwrap_or_else(|_| "http://localhost:8000".into());
    let token = std::env::var("OGL_TOKEN").ok();
    let client = Client::new(url, token)?;

    // 1. Crée une collection.
    let collection_id = client
        .create_collection(&CollectionRequest::new("demo-aides"))
        .await?;
    println!("collection #{collection_id}");

    // 2. Ingère un document (multipart).
    let doc = client
        .create_document(
            "aide.txt",
            "Subvention pour le matériel d'irrigation économe en eau."
                .as_bytes()
                .to_vec(),
            &CreateDocument::new(collection_id),
        )
        .await?;
    println!("document #{}", doc.id);

    // 3. Recherche dans la collection.
    let results = client
        .search(
            &CreateSearch::new("aide à l'irrigation")
                .collections([collection_id])
                .limit(3),
        )
        .await?;
    for r in &results.data {
        println!("  score={:.3} : {}", r.score, r.chunk.content);
    }
    Ok(())
}
