use std::{fmt::Write, sync::Arc};

use axum::{extract::State, routing::get, Router};
use sysinfo::System;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {system: Mutex::new(System::new())});


    let app = Router::new()
        .route("/", get(root_get))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7032").await.unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    
    axum::serve(listener, app).await.unwrap();
}

struct AppState {
    system: Mutex<System>
}

async fn root_get(State(state): State<Arc<AppState>>) -> String {
    let mut s = String::new(); 
    
    let mut sys = state.system.lock().await;
    sys.refresh_all();

    writeln!(&mut s, "System name: {:?}", System::name()).unwrap();
    writeln!(&mut s, "memory used: {}%", 100 * sys.used_memory() / sys.total_memory()).unwrap();
    writeln!(&mut s, "total memory: {}", sys.total_memory()).unwrap();
    writeln!(&mut s, "CPUs: {}", sys.cpus().len()).unwrap();

    for (i, cpu) in sys.cpus().iter().enumerate() {
        writeln!(&mut s, "\tCPU {: >2}: {}%", i+1, cpu.cpu_usage()).unwrap();
    }

    s
}