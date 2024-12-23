use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    PgPool,
};

#[derive(Clone)]
struct MyState {
    pool: PgPool,
    token_map: Arc<Mutex<HashMap<String, i64>>>,
}

#[derive(Deserialize)]
struct Payload {
    author: String,
    quote: String,
}

#[derive(FromRow, Serialize)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

#[derive(Serialize)]
struct Quotes {
    quotes: Vec<Quote>,
    page: i64,
    next_token: Option<String>,
}
#[derive(Debug, Deserialize)]
struct ListQuery {
    token: String,
}

fn uuid_from_str(s: &str) -> Result<Uuid, StatusCode> {
    Uuid::from_str(s).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn reset(State(state): State<MyState>) {
    sqlx::query("DELETE FROM quotes")
        .execute(&state.pool)
        .await
        .unwrap();
}

async fn cite(
    State(state): State<MyState>,
    Path(id): Path<String>,
) -> Result<Json<Quote>, StatusCode> {
    let id = uuid_from_str(&id)?;
    sqlx::query_as(
        r#"
        SELECT id, author, quote, created_at, version
        FROM quotes
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::NOT_FOUND)
}

async fn remove(
    State(state): State<MyState>,
    Path(id): Path<String>,
) -> Result<Json<Quote>, StatusCode> {
    let id = uuid_from_str(&id)?;
    sqlx::query_as(
        r#"
        DELETE FROM quotes
        WHERE id = $1
        RETURNING id, author, quote, created_at, version
        "#,
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::NOT_FOUND)
}

async fn undo(
    State(state): State<MyState>,
    Path(id): Path<String>,
    Json(payload): Json<Payload>,
) -> Result<Json<Quote>, StatusCode> {
    let id = uuid_from_str(&id)?;
    sqlx::query_as(
        r#"
        UPDATE quotes
        SET author = $1, quote = $2, version = version+1
        WHERE id = $3
        RETURNING id, author, quote, created_at, version
        "#,
    )
    .bind(payload.author)
    .bind(payload.quote)
    .bind(&id)
    .fetch_one(&state.pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::NOT_FOUND)
}

async fn draft(
    State(state): State<MyState>,
    Json(payload): Json<Payload>,
) -> (StatusCode, Json<Quote>) {
    let quote: Quote = sqlx::query_as(
        r#"
        INSERT INTO quotes (id, author, quote)
        VALUES ($1, $2, $3)
        RETURNING id, author, quote, created_at, version
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.author)
    .bind(payload.quote)
    .fetch_one(&state.pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(quote))
}

fn gen_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

async fn list(
    State(state): State<MyState>,
    query: Option<Query<ListQuery>>,
) -> Result<Json<Quotes>, StatusCode> {
    let page_number = if let Some(Query(query)) = query {
        let mut map = state.token_map.lock().unwrap();
        let number = map
            .get(&query.token)
            .map(|i| *i)
            .ok_or(StatusCode::BAD_REQUEST)?;
        map.remove(&query.token);
        number
    } else {
        0
    };

    let offset = page_number * 3;

    let (count,): (i64,) = sqlx::query_as(r"SELECT COUNT(id) FROM quotes")
        .fetch_one(&state.pool)
        .await
        .unwrap();

    let next_token = if offset + 3 >= count {
        None
    } else {
        let token = gen_random_string();
        state
            .token_map
            .lock()
            .unwrap()
            .insert(token.clone(), page_number + 1);
        Some(token)
    };

    let quotes = sqlx::query_as(
        r#"
        SELECT id, author, quote, created_at, version
        FROM quotes
        ORDER BY created_at ASC
        LIMIT 3 OFFSET $1
        "#,
    )
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(Quotes {
        quotes,
        page: page_number + 1,
        next_token,
    }))
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/19/reset", post(reset))
        .route("/19/cite/:id", get(cite))
        .route("/19/remove/:id", delete(remove))
        .route("/19/undo/:id", put(undo))
        .route("/19/draft", post(draft))
        .route("/19/list", get(list))
        .with_state(MyState {
            pool,
            token_map: Arc::new(Mutex::new(HashMap::new())),
        })
}