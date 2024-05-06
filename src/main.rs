use std::sync::Arc;

use axum::{extract::{Path, State}, http::{header::{self, CONTENT_TYPE}, HeaderValue, Method, Response, StatusCode}, response::{Html, IntoResponse}, routing::get, Json, Router};
use sysinfo::System;
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::Mutex;


#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {system: Mutex::new(System::new())});


    let app = Router::new()
        .route("/", get(index_html_get))
        .route("/index.mjs", get(index_mjs_get))
        .route("/api/cpus", get(cpus_get))
        .route("/static/*path", get(static_get))   
        .with_state(shared_state)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_private_network(true)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([CONTENT_TYPE])
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7032").await.unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    
    axum::serve(listener, app).await.unwrap();
}

struct AppState {
    system: Mutex<System>
}

#[axum::debug_handler]
async fn static_get(Path(path): Path<String>) -> impl IntoResponse {
    let path = &format!("web/static/{}", path.trim_start_matches('/'));
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    print!("[GET] STATIC: Fetching {path} -> ");

    match tokio::fs::read_to_string(path).await {
        Err(err) => {
            println!("Error: {}", err);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(format!("Error: {err}"))
                .unwrap()
        },
        Ok(markup) => {
            println!("Success! MIME Type: {:?}", HeaderValue::from_str(mime_type.as_ref()).unwrap());
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, HeaderValue::from_str(mime_type.as_ref()).unwrap(),)
                .body(markup)
                .unwrap()
        }
    }
}

#[axum::debug_handler]
async fn index_html_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("web/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn index_mjs_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("web/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn cpus_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut sys = state.system.lock().await;
    sys.refresh_cpu();
    sys.refresh_memory();
    
    let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    Json((System::name(), sys.total_memory(), sys.used_memory(), sys.cpus().len(), v, System::host_name()))
}