use std::sync::Arc;
use std::env;
pub use self::error::{Error, Result};

use crate::ctx::Ctx;
use crate::log::log_request;
use crate::model::manager::ModelManager;
use axum::extract::{Path, Query};
use axum::http::{header, HeaderValue, Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{middleware, Json, Router};
use axum_prometheus::PrometheusMetricLayer;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;
use crate::config::config;
use tower_http::cors::{Any, CorsLayer};
mod ctx;
mod error;
mod log;
pub mod model;

pub mod web;
mod open_telemetry;
pub mod dev_utils;
mod config;

mod crypt;
mod utils;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = open_telemetry::init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();


    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let metric_handle = Arc::new(metric_handle);
    let config = config();

    //dev_utils::init_dev().await;

    let mm = ModelManager::new(config).await?;

    let routes_apis = Router::new()
        .merge(web::routes_user::routes(mm.clone()))
        .merge(web::routes_task::routes(mm.clone()))
        .merge(web::routes_document::routes(mm.clone()))
        .merge(web::routes_query_data::routes(mm.clone()))
        .merge(web::routes_fine_tune::routes(mm.clone()))
        .merge(web::routes_chat::routes(mm.clone()))
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_admin = Router::new()
        .merge(web::routes_statistics::routes(mm.clone()))
        .route(
            "/metrics",
            get({
                let metric_handle = Arc::clone(&metric_handle);
                move || {
                    let metric_handle = Arc::clone(&metric_handle);
                    async move { metric_handle.render() }
                }
            })
        )
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_admin));

    let routes_metrics = Router::new()
        .route(
            "/metrics",
            get({
                let metric_handle = Arc::clone(&metric_handle);
                move || {
                    let metric_handle = Arc::clone(&metric_handle);
                    async move { metric_handle.render() }
                }
            })
        );

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_register::routes(mm.clone()))
        .merge(web::routes_login::routes(mm.clone()))
        .nest("/api", routes_apis)
        .nest("/admin", routes_admin)
        .merge(routes_metrics)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .layer(prometheus_layer)
        .fallback_service(routes_static());

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3100".parse::<HeaderValue>().unwrap())
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            header::CONTENT_TYPE,         // JSON POSTs from the browser
            header::ACCEPT,               // fetch default
            header::AUTHORIZATION,        // if you ever add bearer tokens
        ])
        .allow_credentials(true);

    let app = routes_all.layer(cors);

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("->> LISTENING on {:?}\n", listener.local_addr());
    info!("Server started at {:?}", listener.local_addr());
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    let error_response =
        client_status_error
            .as_ref()
            .map(|(status_code, client_error)| {
                let client_error_body = json!({
                    "error": {
                        "type": client_error.as_ref(),
                        "req_uuid": uuid.to_string(),
                    }
                });

                println!("    ->> client_error_body: {client_error_body}");

                (*status_code, Json(client_error_body)).into_response()
            });

    let client_error = client_status_error.unzip().1;
    let _ =
        log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello2 <strong>{name}</strong>"))
}
