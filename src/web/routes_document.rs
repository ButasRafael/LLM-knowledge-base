use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json, Router,
    routing::{delete, get, post, put},
};
use futures_util::{StreamExt, TryFutureExt};
use sanitize_filename::sanitize;
use serde::Deserialize;
use std::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    ctx::Ctx,
    error::{Error, Result},
    model::documents::{
        Document, DocumentBmc, DocumentForCreateInternal, DocumentForUpdate
    },
    model::manager::ModelManager,
};

const MAX_FILE_SIZE_BYTES: usize = 50 * 1024 * 1024;

#[instrument]
pub async fn upload_documents(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    mut multipart: Multipart,
) -> Result<Json<Vec<Document>>> {
    println!("->> {:<12} - upload_documents (chunk-based)", "HANDLER");

    tokio::fs::create_dir_all("/usr/local/bin/uploads")
        .await
        .map_err(|_| Error::DocumentUploadFail)?;

    let mut uploaded_docs = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::DocumentUploadFail)?
    {
        let filename_opt = field.file_name();
        if filename_opt.is_none() {
            let desc_text = field.text().await.unwrap_or_default();
            println!("Got a 'description' field: {desc_text}");
            continue;
        }

        let original_filename = filename_opt.unwrap().to_string();
        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        info!("Processing file: {original_filename}, type: {content_type}");

        let allowed_types = ["application/pdf", "text/plain", "text/markdown"];
        if !allowed_types.contains(&content_type.as_str()) {
            return Err(Error::ServiceError(format!(
                "Unsupported Content-Type: {content_type}"
            )));
        }

        let unique_id = Uuid::new_v4();
        let sanitized_name = sanitize(&original_filename);
        let filepath = format!("/usr/local/bin/uploads/{}_{}", unique_id, sanitized_name);

        let mut file = File::create(&filepath)
            .await
            .map_err(|_| Error::DocumentUploadFail)?;

        let mut total_bytes = 0usize;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| Error::DocumentUploadFail)?
        {
            total_bytes += chunk.len();
            if total_bytes > MAX_FILE_SIZE_BYTES {
                return Err(Error::ServiceError(format!(
                    "File too large: {} bytes (limit: {} bytes)",
                    total_bytes, MAX_FILE_SIZE_BYTES
                )));
            }

            file.write_all(&chunk)
                .await
                .map_err(|_| Error::DocumentUploadFail)?;
        }


        let doc_internal = DocumentForCreateInternal {
            filename: original_filename.clone(),
            filepath: filepath.clone(),
            uploaded_by: ctx.user_id(),
        };

        let document = DocumentBmc::upload_document_with_embedding(&ctx, &mm, doc_internal).await?;
        uploaded_docs.push(document);
    }

    if uploaded_docs.is_empty() {
        return Err(Error::DocumentUploadFail);
    }

    Ok(Json(uploaded_docs))
}

#[tracing::instrument]
pub async fn list_documents(
    State(mm): State<ModelManager>,
    ctx: Ctx,
) -> Result<Json<Vec<Document>>> {
    println!("->> {:<12} - list_documents", "HANDLER");
    let documents = DocumentBmc::list(&ctx, &mm).await?;
    info!("Documents listed");
    Ok(Json(documents))
}

#[tracing::instrument]
pub async fn get_document(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<Document>> {
    println!("->> {:<12} - get_document", "HANDLER");
    let document = DocumentBmc::get(&ctx, &mm, id).await?;
    info!("Document retrieved: {:?}", document);
    Ok(Json(document))
}

#[tracing::instrument]
pub async fn update_document(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
    Json(doc_u): Json<DocumentForUpdate>,
) -> Result<StatusCode> {
    println!("->> {:<12} - update_document", "HANDLER");
    DocumentBmc::update(&ctx, &mm, id, doc_u).await?;
    info!("Document updated: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument]
pub async fn delete_document(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    println!("->> {:<12} - delete_document", "HANDLER");
    DocumentBmc::delete(&ctx, &mm, id).await?;
    info!("Document deleted: id={}", id);
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/documents/upload", post(upload_documents))
        .route("/documents", get(list_documents))
        .route("/documents/:id", get(get_document))
        .route("/documents/:id", put(update_document))
        .route("/documents/:id", delete(delete_document))
        .with_state(mm)
}
