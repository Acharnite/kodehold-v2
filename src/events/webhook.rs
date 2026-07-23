use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

use crate::events::{Dispatcher, Event, EventSource};

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookState {
    pub dispatcher: Dispatcher,
    pub secret: Option<String>,
}

pub fn router(dispatcher: Dispatcher, secret: Option<String>) -> Router {
    let state = Arc::new(WebhookState { dispatcher, secret });

    Router::new()
        .route("/hooks/github", post(handle_github))
        .route("/health", post(|| async { StatusCode::OK }))
        .with_state(state)
}

async fn handle_github(
    State(state): State<Arc<WebhookState>>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, &'static str) {
    if let Some(ref secret) = state.secret {
        let sig_header = match headers.get("x-hub-signature-256") {
            Some(v) => v.to_str().unwrap_or(""),
            None => return (StatusCode::UNAUTHORIZED, "missing signature"),
        };

        if !verify_signature(secret, &body, sig_header) {
            return (StatusCode::UNAUTHORIZED, "invalid signature");
        }
    }

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let payload: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid JSON"),
    };

    let event = Event::new(EventSource::GitHubWebhook, event_type, payload);

    match state.dispatcher.dispatch(&event) {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(e) => {
            eprintln!("webhook dispatch error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "dispatch error")
        }
    }
}

fn verify_signature(secret: &str, body: &str, signature: &str) -> bool {
    let expected_prefix = "sha256=";
    if !signature.starts_with(expected_prefix) {
        return false;
    }
    let sig_hex = &signature[expected_prefix.len()..];

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body.as_bytes());
    let code = mac.finalize().into_bytes();

    let computed = hex::encode(code);
    computed == sig_hex
}
