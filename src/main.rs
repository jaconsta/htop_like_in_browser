use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router, Server,
};
use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let app_state = AppState {
        sys: Arc::new(Mutex::new(System::new())),
    };

    let router = Router::new();
    let router = router.route("/healthcheck", get(healthcheck));
    let router = router
        .route("/", get(public_serve_get))
        .route("/index.js", get(js_serve_get))
        .route("/index.mjs", get(mjs_serve_get))
        .route("/index.css", get(css_serve_get));
    let router = router
        .route("/api/cpus/string", get(proc_as_string_get))
        .route("/api/cpus/json", get(proc_as_vec_get))
        .with_state(app_state);
    let server = Server::bind(&"0.0.0.0:8082".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr().to_string();
    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn healthcheck() -> &'static str {
    "I am alive"
}
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
    // let markup = tokio::fs::read_to_string("src/public/index.js")
    //     .await
    //     .unwrap();
    // Response::builder()
    //     .header("content-type", "application/javascript;charset=utf-8")
    //     .body(markup)
    //     .unwrap()
}
#[axum::debug_handler]
async fn mjs_serve_get() -> impl IntoResponse {
    xjs_serve_get("index.mjs").await
}
async fn css_serve_get() -> impl IntoResponse {
    csss_serve_get("index.css").await
}

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
}

async fn proc_as_string_get(State(state): State<AppState>) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();
    for (i, cpu) in sys.cpus().iter().enumerate() {
        let i = i + 1;
        let usage = cpu.cpu_usage();
        writeln!(&mut s, "CPU {i} {usage}%").unwrap();
    }

    s
}

async fn proc_as_vec_get(State(state): State<AppState>) -> impl IntoResponse {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();

    let cpus: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    Json(cpus)
}
