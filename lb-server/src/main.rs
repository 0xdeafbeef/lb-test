use axum::http::HeaderMap;
use axum::response::IntoResponse;
use std::borrow::Cow;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {
    let my_ip = reqwest::get("http://ifconfig.me")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let my_ip = my_ip.trim().to_owned();

    let ctx = Arc::new(Ctx::new(my_ip));
    let config = RustlsConfig::from_pem_file(
        "cert.pem",
        "key.pem",
    )
        .await
        .unwrap();
    let app = axum::Router::new()
        .route("/", axum::routing::get(big_answer))
        .route("/stats", axum::routing::get(stats))
        .with_state(ctx);
    println!("Listening on 3000");
    axum_server::bind_rustls("0.0.0.0:3000".parse().unwrap(), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct Ctx {
    num_requests: AtomicUsize,
    my_ip: String,
}

impl Ctx {
    fn new(my_ip: String) -> Self {
        Self {
            num_requests: AtomicUsize::new(0),
            my_ip,
        }
    }
}

async fn big_answer(
    axum::extract::State(ctx): axum::extract::State<Arc<Ctx>>,
) -> impl IntoResponse {
    const ANSWER: &str =
        "The answer to the ultimate question of life, the universe and everything is 42!\n";
    const ANSWER_BIG: &str = const_str::repeat!(ANSWER, 100);
    ctx.num_requests
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut headers = HeaderMap::new();
    headers.insert("X-My-IP", ctx.my_ip.parse().unwrap());
    (headers, Cow::Borrowed(ANSWER_BIG))
}

async fn stats(axum::extract::State(ctx): axum::extract::State<Arc<Ctx>>) -> String {
    format!(
        "Requests: {}",
        ctx.num_requests.load(std::sync::atomic::Ordering::Relaxed)
    )
}
