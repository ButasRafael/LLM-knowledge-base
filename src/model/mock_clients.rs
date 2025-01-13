
use crate::error::{Result};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct SwiftideClient {
    pub qdrant_client: QdrantClient,
}

#[derive(Clone, Debug)]
pub struct OllamaClient;

#[derive(Clone, Debug)]
pub struct QdrantClient {
    embeddings: Arc<Mutex<HashMap<u64, Vec<f32>>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

impl SwiftideClient {
    pub fn new(qdrant_client: QdrantClient) -> Self {
        Self { qdrant_client }
    }

    pub async fn create_embedding(&self, text: &str) -> Result<Vec<f32>> {
        println!("SwiftideClient: Creating embedding for text: {}", text);
        // Simulate processing time.
        sleep(Duration::from_millis(100)).await;
        // Return a mock embedding.
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    pub async fn semantic_search(&self, query_embedding: &[f32]) -> Result<Vec<String>> {
        println!(
            "SwiftideClient: Performing semantic search with embedding: {:?}",
            query_embedding
        );

        sleep(Duration::from_millis(100)).await;


        let _embeddings = self.qdrant_client.get_all_embeddings().await?;

        Ok(vec![
            "Document 1 content related to query".to_string(),
            "Document 2 content related to query".to_string(),
        ])
    }
}

impl QdrantClient {
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn store_embedding(&self, document_id: i64, embedding: Vec<f32>) -> Result<()> {
        println!("QdrantClient: Storing embedding for document ID: {}", document_id);
        let mut embeddings = self.embeddings.lock().await;
        embeddings.insert(document_id as u64, embedding);
        Ok(())
    }

    pub async fn get_all_embeddings(&self) -> Result<HashMap<u64, Vec<f32>>> {
        let embeddings = self.embeddings.lock().await;
        Ok(embeddings.clone())
    }
}

impl OllamaClient {

    pub fn new() -> Self {
        Self
    }

    pub async fn get_response(&self, prompt: &str) -> Result<String> {
        println!("OllamaClient: Generating response for prompt: {}", prompt);
        sleep(Duration::from_millis(100)).await;
        Ok(format!("Mock response to: {}", prompt))
    }
}
