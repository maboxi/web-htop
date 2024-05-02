use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use sysinfo::System;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {system: Mutex::new(System::new())});


    let app = Router::new()
        .route("/", get(root_get))
        .route("/api/cpus", get(cpus_get))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7032").await.unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    
    axum::serve(listener, app).await.unwrap();
}

struct AppState {
    system: Mutex<System>
}

async fn root_get() -> &'static str {
    "Hello there"
}

#[axum::debug_handler]
async fn cpus_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut sys = state.system.lock().await;
    sys.refresh_all();
    
    let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    let memusage = 100 * sys.used_memory() / sys.total_memory();

    Json((System::name(), memusage, sys.total_memory(), sys.cpus().len(), v))
}