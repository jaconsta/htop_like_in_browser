use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router, Server,
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;

type Snapshot = Vec<f32>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (tx, _) = broadcast::channel::<Snapshot>(1);
    let app_state = AppState { tx: tx.clone() };

    let router = Router::new();
    let router = router.route("/healthcheck", get(healthcheck));
    let router = router
        .route("/", get(public_serve_get))
        .route("/index.js", get(js_serve_get))
        .route("/index.mjs", get(mjs_serve_get))
        .route("/index.css", get(css_serve_get));
    let router = router
        .route("/api/cpus/string", get(cpu_as_string_get))
        .route("/api/cpus/json", get(cpu_as_vec_get))
        .route("/ws/cpus/json", get(cpu_as_vec_ws_get))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let cpus: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let _ = tx.send(cpus);
            // Min interval comes from System crate
            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(&"0.0.0.0:8082".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr().to_string();
    println!("Listening on {addr}");

    server.await.unwrap();
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Snapshot>,
}

async fn healthcheck() -> &'static str {
    "I am alive"
}

// Static file serve
#[axum::debug_handler]
async fn public_serve_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/public/index.html")
        .await
        .unwrap();
    Html(markup)
}

async fn xxx_serve_get(filename: &str, content_type: &str) -> impl IntoResponse {
    let markup = tokio::fs::read_to_string(format!("src/public/{filename}"))
        .await
        .unwrap();
    Response::builder()
        .header("content-type", format!("{content_type};charset=utf-8"))
        .body(markup)
        .unwrap()
}
async fn xjs_serve_get(filename: &str) -> impl IntoResponse {
    xxx_serve_get(filename, "application/javascript").await
}
async fn csss_serve_get(filename: &str) -> impl IntoResponse {
    xxx_serve_get(filename, "text/css").await
}
#[axum::debug_handler]
async fn js_serve_get() -> impl IntoResponse {
    xjs_serve_get("index.js").await
}
#[axum::debug_handler]
async fn mjs_serve_get() -> impl IntoResponse {
    xjs_serve_get("index.mjs").await
}
async fn css_serve_get() -> impl IntoResponse {
    csss_serve_get("index.css").await
}

// Api endpoints
async fn cpu_as_string_get(State(state): State<AppState>) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let mut rx = state.tx.subscribe();
    if let Ok(cpus) = rx.recv().await {
        for (i, cpu) in cpus.iter().enumerate() {
            let i = i + 1;
            writeln!(&mut s, "CPU {i} {cpu}%").unwrap();
        }
    }

    s
}

async fn cpu_as_vec_get(State(state): State<AppState>) -> impl IntoResponse {
    // let lock_start = std::time::Instant::now();
    let mut rx = state.tx.subscribe();
    let cpus = rx.recv().await.unwrap_or_else(|_| vec![]);
    // println!("Lock time: {}ms", lock_start.elapsed().as_millis());
    Json(cpus)
}

async fn cpu_as_vec_ws_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { cpus_get_stream(state, ws).await })
}

async fn cpus_get_stream(state: AppState, mut ws: WebSocket) {
    let mut rx = state.tx.subscribe();
    while let Ok(cpus) = rx.recv().await {
        let payload = serde_json::to_string(&cpus).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}
