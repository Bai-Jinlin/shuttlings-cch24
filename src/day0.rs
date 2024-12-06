use axum::{http::{HeaderMap, StatusCode}, routing::{get, Router}};

async fn p1()->String{
    "Hello, bird!".into()
}

async fn p2()->(HeaderMap,StatusCode){
    let mut headers = HeaderMap::new();
    headers.insert("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4".parse().unwrap());
    (headers,StatusCode::FOUND)
}

pub fn router()->Router{
    Router::new().route("/", get(p1)).route("/-1/seek", get(p2))
}