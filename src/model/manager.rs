
use crate::{Error, Result};
use crate::config::Config;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use async_openai::Client as OpenAIClient;

use swiftide::{integrations::{
    qdrant::Qdrant,
    redis::Redis,
    ollama::Ollama,
}, query};
use swiftide::indexing::EmbeddedField;
use swiftide::indexing::transformers::{metadata_keywords, metadata_qa_text, metadata_summary, metadata_title};
use swiftide::integrations::ollama::config::OllamaConfig;
use swiftide::integrations::qdrant::{Distance, VectorConfig};
use swiftide::query::{answers, query_transformers, response_transformers};
use tracing::instrument;
use crate::ctx::Ctx;

pub type Db = Pool<Postgres>;

#[derive(Debug, Clone)]
pub struct ModelManager {
    pub db: Db,
    pub qdrant: Qdrant,
    pub redis_cache: Redis,
    pub ollama: Ollama,
}

impl ModelManager {
    pub async fn new(config: &Config) -> Result<Self> {

        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.DB_URL)
            .await
            .map_err(|e| Error::FailToCreatePool(e.to_string()))?;

        let redis_cache = Redis::try_from_url(&config.REDIS_URL, "knowledge-base")
            .map_err(|e| Error::RedisError(e.to_string()))?;

        let qdrant = Qdrant::try_from_url(&config.QDRANT_URL).
            map_err(|e| Error::QdrantError(e.to_string()))?
            .batch_size(50)
            .vector_size(4096)
            .collection_name("knowledge-base")
            .with_vector(EmbeddedField::Combined)
            .with_vector(EmbeddedField::Chunk)
            .with_vector(EmbeddedField::Metadata(metadata_qa_text::NAME.into()))
            .with_vector(EmbeddedField::Metadata(metadata_summary::NAME.into()))
            .with_vector(VectorConfig::builder()
                .embedded_field(EmbeddedField::Metadata(metadata_title::NAME.into()))
                .distance(Distance::Manhattan).build()?)
            .with_vector(EmbeddedField::Metadata(metadata_keywords::NAME.into()))
            .build().map_err(|e| Error::QdrantError(e.to_string()))?;

        let mut cfg = OllamaConfig::default();
        cfg.with_api_base("http://ollama:11434/v1");
        let custom_client = OpenAIClient::with_config(cfg);

        let ollama = Ollama::builder()
            .client(custom_client)
            .default_prompt_model("llama3.1")
            .default_embed_model("llama3.1")
            .build()?;

        Ok(Self {
            db,
            qdrant,
            redis_cache,
            ollama,
        })
    }

    #[instrument(skip_all, name = "ModelManager.query_data")]
    pub async fn query_data(&self, _ctx: &Ctx, prompt: &str) -> Result<Vec<String>> {
        let pipeline = query::Pipeline::default()
            .then_transform_query(query_transformers::GenerateSubquestions::from_client(
                self.ollama.clone(),
            ))
            .then_transform_query(query_transformers::Embed::from_client(
                self.ollama.clone(),
            ))
            .then_retrieve(self.qdrant.clone())
            .then_transform_response(response_transformers::Summary::from_client(
                self.ollama.clone(),
            ))
            .then_answer(answers::Simple::from_client(self.ollama.clone()));

        let result = pipeline.query(prompt).await.map_err(|e| Error::QueryError(e.to_string()))?;

        let documents = result.documents().iter().map(|doc| doc.metadata().get("Title").unwrap().to_string()).collect();
        Ok(documents)
    }

    #[instrument(skip_all, name = "ModelManager.fine_tune_prompt")]
    pub async fn fine_tune_prompt(
        &self,
        prompt: &str,
    ) -> Result<String> {

        let pipeline = query::Pipeline::default()
            .then_transform_query(query_transformers::GenerateSubquestions::from_client(
                self.ollama.clone(),
            ))
            .then_transform_query(query_transformers::Embed::from_client(
                self.ollama.clone(),
            ))
            .then_retrieve(self.qdrant.clone())
            .then_transform_response(response_transformers::Summary::from_client(
                self.ollama.clone(),
            ))
            .then_answer(answers::Simple::from_client(self.ollama.clone()));

        let result = pipeline.query(prompt).await.map_err(|e| Error::QueryError(e.to_string()))?;
        Ok(result.answer().to_string())
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
