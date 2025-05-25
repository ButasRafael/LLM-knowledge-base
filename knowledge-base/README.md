# Knowledge Base Project

A full-stack application for managing users, tasks, documents, and AI-powered chat queries. The project is divided into two main parts:

* **Backend** (`knowledge-base/`): A Rust-based server using Axum, SQLx, Sea-Query, ModQL, and Swiftide integrations for database operations, document indexing, and AI pipelines.
* **Frontend** (`knowledge-base-ui/`): A Next.js + React UI with shadcn/ui components, providing user authentication, document upload, task management, and chat interfaces.

---

## Table of Contents

1. [Backend Overview](#backend-overview)

   * [Key Generation CLI](#key-generation-cli)
   * [Database Initialization](#database-initialization)
   * [Schema Definitions](#schema-definitions)
   * [Encryption & Authentication](#encryption--authentication)
   * [Database Access Layer](#database-access-layer)
   * [Model Manager & AI Integrations](#model-manager--ai-integrations)
   * [API Routes & Middleware](#api-routes--middleware)
   * [Configuration](#configuration)
   * [Main Entry Point](#main-entry-point)
2. [Frontend Overview](#frontend-overview)
3. [Getting Started](#getting-started)
4. [License](#license)

---

## Backend Overview

The backend is structured as a single Axum-based Rust crate under `knowledge-base/`. It provides:

* User management (registration, login, roles)
* Task and document CRUD operations
* AI-powered document embedding, retrieval, and chat
* Real-time statistics and metrics

All database operations use **SQLx** with **Sea-Query** builders and **ModQL** for ergonomic inserts/updates. Swiftide handles document embedding pipelines, connected to Qdrant, Redis, and Ollama.

### Key Generation CLI

A small CLI (`examples/gen_key.rs`) generates HMAC keys for passwords and tokens:

```rust
use anyhow::Result;
use rand::RngCore;

fn main() -> Result<()> {
    // Generate 64‑byte random key
    let mut key = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut key);
    println!("Generated key: {key:?}");

    // Base64URL encode
    println!("Key b64u encoded: {}", base64_url::encode(&key));
    Ok(())
}
```

### Database Initialization

`dev_utils/init_dev_db.rs` runs on startup (in dev mode) to:

1. Terminate existing connections and recreate the `app_db` and `app_user`.
2. Execute SQL scripts in `sql/dev_initial/`, including schema creation and seed data.
3. Reset the `admin` user password to a known dev value.

Example SQL snippet:

```sql
CREATE USER app_user WITH PASSWORD 'dev_only_pwd';
CREATE DATABASE app_db OWNER app_user;

-- tables:
CREATE TABLE "user" (...);
CREATE TABLE task(...);
CREATE TABLE document(...);
CREATE TABLE conversation(...);
CREATE TABLE message(...);
CREATE TABLE pipeline_log(...);

INSERT INTO "user" (id, username, role) VALUES (1, 'admin', 'admin');
```

### Schema Definitions

Each table has an accompanying Rust model in `src/model/*`, using `#[derive(Fields, FromRow)]`. Sea-Query auto-generates SQL under the hood:

* **User**: id, username, pwd hash, salts, role
* **Task**: id, title, created\_by
* **Document**: id, filename, filepath, uploaded\_by
* **Conversation**/**Message**: chat history for AI chat
* **PipelineLog**: metrics for each AI pipeline run

### Encryption & Authentication

Custom HMAC-SHA512-based encryption in `src/crypt/`:

* `encrypt_pwd()` hashes passwords with per-user salt and a global `PWD_KEY`.
* JWT-like `Token` struct signs user identifier and expiration with a dedicated `TOKEN_KEY`.
* Cookies store an `auth-token` on login; middleware validates and refreshes it each request.

### Database Access Layer

`src/model/base.rs` provides generic CRUD via Sea-Query + SQLx:

```rust
pub async fn create<MC, E>(...) { /* insert and return id */ }
pub async fn get<MC, E>(...) { /* select by id */ }
// list, update, delete similarly
```

Each resource (e.g. `UserBmc`, `TaskBmc`, `DocumentBmc`) wraps these base functions for its domain.

### Model Manager & AI Integrations

`ModelManager` in `src/model/manager.rs` centralizes:

* **Postgres** pool via SQLx
* **Redis** cache for embeddings
* **Qdrant** vector store configured with Combined, Chunk, and Metadata vectors
* **Ollama** client for both embedding and chat, using GPU-enabled models

It exposes two main Swiftide-powered pipelines:

1. **Embedding Pipeline** (`upload_document_with_embedding` in `DocumentBmc`):

   * Reads the uploaded file (Markdown, text, or PDF)
   * Splits content into chunks (`ChunkMarkdown` or `ChunkText`)
   * Enriches each chunk with metadata (Q\&A prompts, summaries, titles, keywords)
   * Performs batched embeddings using the dedicated embedding model (`bge-m3`)
   * Caches results in Redis, then stores vectors in Qdrant for fast retrieval

2. **Retrieval-and-Query Pipeline** (`query_data` and `fine_tune_prompt` in `ModelManager`):

   * **Query transformation**: generates sub-questions and embeds the user prompt via Ollama
   * **Vector retrieval**: fetches relevant chunks from Qdrant
   * **Response transformation**: summarizes retrieved contexts
   * **Answer generation**: produces a final answer with the chat model
   * Logs pipeline duration in `pipeline_log` for monitoring and analytics

This Swiftide integration enables a robust RAG (Retrieval-Augmented Generation) workflow: ingest and index documents with rich embeddings, then power real-time AI-driven query and chat experiences.

### API Routes & Middleware

Routes in `src/web/` are organized by resource:

* `routes_user.rs`: `/api/users`, `/api/users/:id/password`, `/api/users/me`
* `routes_task.rs`, `routes_document.rs`, `routes_chat.rs`, etc.
* Global middleware:

  * **Ctx resolver**: extracts and validates auth token from cookies
  * **Role checks**: `mw_require_auth`, `mw_require_admin`
  * **Error mapper**: converts service errors into structured JSON responses

### Configuration

All settings are loaded from environment variables into a singleton `Config` via `src/config.rs`:

```
SERVICE_PWD_KEY          Base64URL HMAC key for passwords
SERVICE_TOKEN_KEY        Base64URL HMAC key for tokens
SERVICE_TOKEN_DURATION_SEC  Token lifetime in seconds
SERVICE_DB_URL           Postgres connection URL
SERVICE_UPLOAD_DIR       Local file upload path
SERVICE_REDIS_URL        Redis URL
SERVICE_QDRANT_URL       Qdrant URL
SERVICE_JAEGER_ENDPOINT  Jaeger OTLP endpoint
SERVICE_OLLAMA_URL       Ollama API base
```

### Main Entry Point

`src/main.rs` wires everything:

1. Initialize tracing and Prometheus metrics
2. Build `ModelManager`
3. Mount API and admin routes with CORS and cookie layers
4. Serve static files and fallback to a router
5. Listen on port 8000

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let mm = ModelManager::new(config()).await?;
    let app = build_router(mm);
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
```

---

## Frontend Overview

*(Brief mention—see `/knowledge-base-ui/README.md` for full details.)*

* **Next.js 14** with App Router
* **Shadcn/ui** and **Tailwind CSS** for layout and components
* **React Context** for auth and API hooks
* **Pages**:

  * Login, Register, Profile
  * Tasks, Documents, Users (admin)
  * Chat interface with conversation list and message threads
  * Metrics & Statistics dashboard

---

## Getting Started

1. Clone the repo and enter the root directory.
2. Copy `.env.example` to `.env` and fill in keys/URLs.
3. Start services:

```
docker-compose up -d
```
4. Build and run the backend:
```
cd knowledge-base
cargo run --release
```

5. Start the frontend:
```
cd knowledge-base-ui
npm install
npm run dev
```
6. Open http://localhost:3100 (frontend) and http://localhost:8000 (backend) in your browser.

---

## License

This project is licensed under the MIT License. Feel free to use, modify, and distribute.

