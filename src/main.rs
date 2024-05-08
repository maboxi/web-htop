use std::sync::{Arc, Mutex};

use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, http::{header::CONTENT_TYPE, Method}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tower_http::cors::{Any, CorsLayer};

const PREFIX_HTOP: &str = "[HTOP]";
const PREFIX_ALGS: &str = "[ALGS]";

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        system_state: Arc::new(Mutex::new(SystemState::default())),
        reqcounter: Mutex::new(0),
        wscounter_cpus: Mutex::new(0),
        wscounter_algs: Mutex::new(0),
    });

    let app = Router::new()
        .route("/api/cpus", get(cpus_get))
        .route("/api/cpus/ws", get(cpus_ws_handler))
        .route("/api/algorithms", post(algorithms_post))
        .route("/api/algorithms/ws/console", get(algorithms_console_ws_handler))
        .with_state(shared_state.clone())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_private_network(true)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([CONTENT_TYPE])
        );

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_all();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

            {
                let mut system_state = shared_state.system_state.lock().unwrap();
                system_state.cpus = sys.cpus().len();
                system_state.cpu_usage = v;
                system_state.total_memory = sys.total_memory();
                system_state.used_memory = sys.used_memory();
                system_state.system_name = System::name().unwrap_or("Unknown".to_string());
                system_state.host_name = System::host_name().unwrap_or("Unknown".to_string());
                system_state.was_updated = true;
            }
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });


    let listener = tokio::net::TcpListener::bind("0.0.0.0:7032").await.unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    
    axum::serve(listener, app).await.unwrap();
}


#[derive(Debug, Serialize, Default, Clone)]
struct SystemState {
    system_name: String,
    total_memory: u64,
    used_memory: u64,
    cpus: usize,
    cpu_usage: Vec<f32>,
    host_name: String,
    was_updated: bool,
}

struct AppState {
    system_state: Arc<Mutex<SystemState>>,
    reqcounter: Mutex<usize>,
    wscounter_cpus: Mutex<usize>,
    wscounter_algs: Mutex<usize>,
}


#[axum::debug_handler]
async fn cpus_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    serde_json::to_string(&(*state.system_state.lock().unwrap())).unwrap_or("{'error': 'system state json conversion failed!'}".to_string())
}

async fn cpus_ws_handler(socket: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    let counter: usize;
    {
        let mut state_counter = state.wscounter_cpus.lock().unwrap();
        *state_counter += 1;
        counter = *state_counter;
    }

    socket.on_upgrade(move |ws| async move { cpus_handle_socket(ws, state, counter).await })
}

async fn cpus_handle_socket(socket: WebSocket, state: Arc<AppState>, wsnum: usize) {
    println!("{PREFIX_HTOP} New cpus websocket connection #{}!", wsnum);
    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(async move {
        loop {
            let payload = serde_json::to_string(&(*state.system_state.lock().unwrap())).unwrap_or("{'error': 'system state json conversion failed!'}".to_string());

            if sender.send(Message::Text(payload)).await.is_err() {
                println!("{PREFIX_HTOP} WS #{wsnum}: Sender closed!");
                return;
           }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    tokio::spawn( async move {    
        loop {
            let msg = receiver.next().await;
            match msg {
                Some(Ok(msg)) => {
                    match msg {
                        Message::Close(None) => {
                            println!("{PREFIX_HTOP} WS #{wsnum}: Received CLOSE message!");
                            return;
                        },
                        msg => {
                            println!("{PREFIX_HTOP} WS #{wsnum}: Received message: {:?}", msg);
                        }
                    }
                },
               Some(Err(e)) => {
                    println!("{PREFIX_HTOP} WS #{wsnum}: Error receiving message: {:?}", e);
                },
                None => {
                    println!("{PREFIX_HTOP} WS #{wsnum}: Receiver closed!");
                    return;
                }
            }
        }
    });
}


/*
    request = {
        "request_type": "list" | "execute",
        "content": {
            <request-specific content>
        }
    }
*/
#[derive(Deserialize)]
struct AlgorithmRequest {
    request_type: AlgorithmRequestType,
    content: serde_json::Value
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmRequestType {
    List,
    Execution
}

#[derive(Deserialize)]
struct AlgorithmListRequest {
    list_type: AlgorithmListType,
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
        let mut state_counter = state.reqcounter.lock().unwrap();
        *state_counter += 1;
        counter = *state_counter;
    }



    let request = match serde_json::from_str::<AlgorithmRequest>(&request_body) {
        Ok(data_tl) => {
            data_tl
        },
        Err(e) => {
            let errmsg = format!("Error parsing toplevel request: {:?}", e);
            println!("{}", errmsg);
            return Json((AlgorithmExecutionResponse::Error, counter, errmsg));
        }
    };

    //println!("Received request #{counter}: {:?}", &request);

    match request.request_type {
        AlgorithmRequestType::List => {
            let request = match serde_json::from_value::<AlgorithmListRequest>(request.content) {
                Ok(content_list) => {
                    content_list
                },
                Err(e) => {
                    let errmsg = format!("Error parsing toplevel request: {:?}", e);
                    println!("{}", errmsg);
                    return Json((AlgorithmExecutionResponse::Error, counter, errmsg));
                }
            };

            handle_algorithm_list_request(request);
        },
        AlgorithmRequestType::Execution => {

        }
    }

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

async fn algorithms_console_ws_handler(socket: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let counter: usize;
    {
        let mut state_counter = state.wscounter_algs.lock().unwrap();
        *state_counter += 1;
        counter = *state_counter;
    }

    socket.on_upgrade(move |ws| async move { algorithms_console_handle_socket(ws, counter).await })
}

async fn algorithms_console_handle_socket(socket: WebSocket, wsnum: usize) {
    println!("{PREFIX_ALGS} New algs websocket connection #{}!", wsnum);
    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            let payload = format!("[{}] alg console test {i}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

            if sender.send(Message::Text(payload)).await.is_err() {
                println!("{PREFIX_ALGS} WS #{wsnum}: Sender closed!");
                return;
            }

            i += 1;
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        }
    });
    tokio::spawn( async move {    
        loop {
            let msg = receiver.next().await;
            match msg {
                Some(Ok(msg)) => {
                    match msg {
                        Message::Close(None) => {
                            println!("{PREFIX_ALGS} WS #{wsnum}: Received CLOSE message!");
                            return;
                        },
                        msg => {
                            println!("{PREFIX_ALGS} WS #{wsnum}: Received message: {:?}", msg);
                        }
                    }
                },
               Some(Err(e)) => {
                    println!("{PREFIX_ALGS} WS #{wsnum}: Error receiving message: {:?}", e);
                },
                None => {
                    println!("{PREFIX_ALGS} WS #{wsnum}: Receiver closed!");
                    return;
                }
            }
        }
    });
}