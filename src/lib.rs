use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use opto_sync_client::{core_version, reconcile, ReconcileOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

pub mod background;

#[derive(Clone)]
pub struct AppState {
    document: Arc<Mutex<Value>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            document: Arc::new(Mutex::new(json!({
                "id": "doc-1",
                "title": "server draft",
                "updatedAt": "100",
                "metadata": {"serverOnly": true}
            }))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
    pub incoming: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub merge_engine: String,
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        merge_engine: core_version().to_string(),
    })
}

async fn document(State(state): State<AppState>) -> Json<Value> {
    Json(state.document.lock().await.clone())
}

async fn merge(
    State(state): State<AppState>,
    Json(request): Json<MergeRequest>,
) -> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let mut document = state.document.lock().await;
    let merged = reconcile(
        &document.to_string(),
        &request.incoming.to_string(),
        &ReconcileOptions::default(),
    )
    .map_err(|error| {
        (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            error.to_string(),
        )
    })?;
    *document = serde_json::from_str(&merged).map_err(|error| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error.to_string(),
        )
    })?;
    Ok(Json(document.clone()))
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/document", get(document))
        .route("/merge", post(merge))
        .with_state(AppState::default())
}
