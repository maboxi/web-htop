use std::sync::Arc;

use axum::{extract::State, http::{header::CONTENT_TYPE, Method}, response::IntoResponse, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tower_http::cors::{Any, CorsLayer};
use tokio::sync::Mutex;


#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {system: Mutex::new(System::new()), reqcounter: Mutex::new(0)});


    let app = Router::new()
        .route("/api/cpus", get(cpus_get))
        .route("/api/algorithms", post(algorithms_post))
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
    system: Mutex<System>,
    reqcounter: Mutex<usize>
}

#[axum::debug_handler]
async fn cpus_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut sys = state.system.lock().await;
    sys.refresh_cpu();
    sys.refresh_memory();
    
    let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    Json((System::name(), sys.total_memory(), sys.used_memory(), sys.cpus().len(), v, System::host_name()))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AlgorithmRequest {
    request_type: AlgorithmRequestType,
    list_type: Option<AlgorithmListType>,
    execution_data: Option<AlgorithmExecutionRequest>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmRequestType {
    List,
    Execution
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmListType {
    All,
    Graph
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AlgorithmExecutionRequest {
    algorithm: String,
    data: String
}


#[derive(Debug, Serialize)]
enum AlgorithmExecutionResponse {
    Ok,
    Error
}

#[axum::debug_handler]
async fn algorithms_post(State(state): State<Arc<AppState>>, request_body: String) -> impl IntoResponse {
    let counter: usize;
    {
        let mut state_counter = state.reqcounter.lock().await;
        *state_counter += 1;
        counter = *state_counter;
    }

    let request: AlgorithmRequest;

    match serde_json::from_str::<AlgorithmRequest>(&request_body) {
        Ok(req) => {
            request = req;
        },
        Err(e) => {
            let errmsg = format!("Error parsing request: {:?}", e);
            println!("{}", errmsg);
            return Json((AlgorithmExecutionResponse::Error, counter, errmsg));
        }
    }

    println!("Received request #{counter}: {:?}", &request);

    let response = match request.request_type {
        AlgorithmRequestType::List => {
            match request.list_type {
                Some(AlgorithmListType::All) => {
                    Json((AlgorithmExecutionResponse::Ok, counter, "['Rucksack-PTAS', 'Rucksack-FPTAS', 'Dijkstra', 'Johnson', 'Prim']".to_string()))
                },
                Some(AlgorithmListType::Graph) => {
                    Json((AlgorithmExecutionResponse::Ok, counter, "Graph algorithms".to_string()))
                },
                None => {
                    Json((AlgorithmExecutionResponse::Error, counter, "No list type specified".to_string()))
                }
            }
        },
        AlgorithmRequestType::Execution => {
            match request.execution_data {
                Some(data) => {
                    Json((AlgorithmExecutionResponse::Ok, counter, format!("{:?}",data)))
                },
                None => {
                    Json((AlgorithmExecutionResponse::Error, counter, "No execution data specified".to_string()))
                }
            }
        }
    };

    response
}