use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let body: Value = match req.body() {
        Body::Text(text) => serde_json::from_str(text)?,
        _ => json!({}),
    };

    let text = body["text"].as_str().unwrap_or("");
    let translation = format!("תרגום הדגמה: {}", text);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::Text(json!({
            "translation": translation
        }).to_string()))?;

    Ok(response)
} 