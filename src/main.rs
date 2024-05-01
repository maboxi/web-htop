use axum::{routing::get, Router};
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root_get));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7032").await.unwrap();
    //let server = Server::bind(&"0.0.0.0:7032".parse().unwrap())
    //    .serve(router.into_make_service());

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root_get() -> &'static str {
    "Hi from my axum site!"
}