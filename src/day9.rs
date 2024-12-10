use std::{sync::Arc, time::Duration};

use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    routing::post,
    Router,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};

type StdMutex<T> = std::sync::Mutex<T>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Payload {
    Liters(f32),
    Litres(f32),
    Gallons(f32),
    Pints(f32),
}
impl Payload {
    fn cal(self) -> Self {
        match self {
            Self::Liters(n) => Self::Gallons(0.264172060 * n),
            Self::Gallons(n) => Self::Liters(3.78541 * n),
            Self::Litres(n) => Self::Pints(1.759754 * n),
            Self::Pints(n) => Self::Litres(0.56826125 * n),
        }
    }
}

fn check_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

    is_json_content_type
}

#[derive(Clone)]
struct MyState {
    limiter: Arc<StdMutex<RateLimiter>>,
}

async fn p1(State(s): State<MyState>, headers: HeaderMap, bytes: Bytes) -> (StatusCode, String) {
    let mut ret = None;
    if check_content_type(&headers) {
        let payload = serde_json::from_slice::<Payload>(&bytes);
        if let Ok(payload) = payload {
            ret = Some((
                StatusCode::OK,
                serde_json::to_string(&payload.cal()).unwrap(),
            ));
        } else {
            ret = Some((StatusCode::BAD_REQUEST, "".to_string()));
        }
    }

    let n = if s.limiter.lock().unwrap().try_acquire(1) {
        (StatusCode::OK, "Milk withdrawn\n".to_string())
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "No milk available\n".to_string(),
        )
    };

    if let Some(ret) = ret {
        ret
    } else {
        n
    }
}
async fn p2(State(s): State<MyState>) {
    *s.limiter.lock().unwrap() = get_default_limiter();
}

fn get_default_limiter() -> RateLimiter {
    RateLimiter::builder()
        .initial(5)
        .max(5)
        .interval(Duration::from_secs(1))
        .refill(1)
        .build()
}

pub fn router() -> Router {
    Router::new()
        .route("/9/milk", post(p1))
        .route("/9/refill", post(p2))
        .with_state(MyState {
            limiter: Arc::new(StdMutex::new(get_default_limiter())),
        })
}
