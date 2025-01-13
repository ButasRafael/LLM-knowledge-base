use std::path::Path;
use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::manager::ModelManager;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;
use tracing::instrument;
use tokio::fs::read_to_string;
use pdf_extract::extract_text;
use tokio::task::spawn_blocking;

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct Document {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub uploaded_by: i64,
}

#[derive(Debug, Fields, Deserialize)]
pub struct DocumentForCreate {
    pub filename: String,
    pub filepath: String,
}

#[derive(Debug, Fields, Deserialize)]
pub struct DocumentForCreateInternal {
    pub filename: String,
    pub filepath: String,
    pub uploaded_by: i64,
}

#[derive(Debug, Fields, Deserialize)]
pub struct DocumentForUpdate {
    pub filename: Option<String>,
    pub filepath: Option<String>,
}

pub struct DocumentBmc;

impl DbBmc for DocumentBmc {
    const TABLE: &'static str = "document";
}

impl DocumentBmc {
    #[instrument]
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        doc_c: DocumentForCreateInternal,
    ) -> Result<Document> {
        let id = base::create::<Self, _>(ctx, mm, doc_c).await?;
        Self::get(ctx, mm, id).await
    }

    #[instrument]
    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Document> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    #[instrument]
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Document>> {
        base::list::<Self, _>(ctx, mm).await
    }
    #[instrument]
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        doc_u: DocumentForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, doc_u).await
    }

    #[instrument]
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
    #[instrument]
    pub async fn upload_document(
        ctx: &Ctx,
        mm: &ModelManager,
        filename: String,
        filepath: String,
    ) -> Result<Document> {
        let doc_internal = DocumentForCreateInternal {
            filename,
            filepath,
            uploaded_by: ctx.user_id(),
        };
        Self::create(ctx, mm, doc_internal).await
    }

    pub async fn parse_pdf_blocking(file_path: String) -> Result<String> {
        let file_path_clone = file_path.clone();

        let parsed_content = spawn_blocking(move || {
            if Path::new(&file_path_clone).exists() {
                extract_text(&file_path_clone).unwrap_or_else(|_| "".to_string())
            } else {
                "".to_string()
            }
        })
            .await
            .map_err(|_| Error::DocumentUploadFail)?;

        Ok(parsed_content)
    }

    #[instrument]
    pub async fn upload_document_with_embedding(
        ctx: &Ctx,
        mm: &ModelManager,
        doc_internal: DocumentForCreateInternal,
    ) -> Result<Document> {
        // Step 1: Upload the document as usual.
        let document = Self::upload_document(ctx, mm, doc_internal.filename.clone(), doc_internal.filepath.clone()).await?;
        let document_id = document.id;
        let document_content: String;
        if doc_internal.filepath.ends_with(".pdf") {
            document_content = Self::parse_pdf_blocking(doc_internal.filepath.clone()).await?;
        } else {
            document_content = read_to_string(&doc_internal.filepath)
                .await
                .map_err(|_| Error::DocumentUploadFail)?;
        }


        let embedding = mm.swiftide_client.create_embedding(&document_content).await?;

        mm.qdrant_client.store_embedding(document_id, embedding).await?;

        Ok(document)
    }
}
