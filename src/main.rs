pub mod algorithms;

use std::sync::{Arc, Mutex};

use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, http::{header::CONTENT_TYPE, Method}, response::{IntoResponse, Response}, routing::{get, post}, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
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


#[axum::debug_handler]
async fn algorithms_post(State(state): State<Arc<AppState>>, request_body: String) -> impl IntoResponse {
    {
        let mut state_counter = state.reqcounter.lock().unwrap();
        *state_counter += 1;
    }

    algorithms::handle_algorithm_request(request_body).await
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