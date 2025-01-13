
use std::time::Duration;
use crate::{Error, Result};
use crate::config::Config;
use sqlx::Pool;
use sqlx::Postgres;
use crate::model::mock_clients::{SwiftideClient, QdrantClient, OllamaClient};
use crate::ctx::Ctx;

pub type Db = Pool<Postgres>;

#[derive(Debug,Clone)]
pub struct ModelManager {
    pub db: Db,
    pub swiftide_client: SwiftideClient,
    pub qdrant_client: QdrantClient,
    pub ollama_client: OllamaClient,
}

impl ModelManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.DB_URL)
            .await
            .map_err(|e| Error::FailToCreatePool(e.to_string()))?;

        let qdrant_client = QdrantClient::new();

        let swiftide_client = SwiftideClient::new(qdrant_client.clone());

        let ollama_client = OllamaClient::new();

        Ok(ModelManager {
            db,
            swiftide_client,
            qdrant_client,
            ollama_client,
        })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn swiftide_client(&self) -> &SwiftideClient {
        &self.swiftide_client
    }

    pub fn qdrant_client(&self) -> &QdrantClient {
        &self.qdrant_client
    }

    pub fn ollama_client(&self) -> &OllamaClient {
        &self.ollama_client
    }

    pub async fn query_data(&self, _ctx: &Ctx, prompt: &str) -> Result<Vec<String>> {
        let prompt_embedding = self.swiftide_client.create_embedding(prompt).await?;

        let retrieved_documents = self.swiftide_client.semantic_search(&prompt_embedding).await?;

        Ok(retrieved_documents)
    }

    pub async fn fine_tune_prompt(&self, prompt: &str, context: &[String]) -> Result<String> {
        let joined_context = context.join("\n");
        let fine_tuned = format!(
            "{prompt}\n\n-- Additional Context --\n{joined_context}"
        );
        Ok(fine_tuned)
    }
}
