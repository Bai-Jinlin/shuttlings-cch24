use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{
    decode, decode_header, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde_json::Value;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

async fn p1(cookies: Cookies, Json(payload): Json<Value>) {
    let jwt = encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(b"asd"),
    )
    .unwrap();
    cookies.add(Cookie::new("gift", jwt));
}
async fn p2(cookies: Cookies) -> Result<Json<Value>, StatusCode> {
    let cookie = cookies.get("gift").ok_or(StatusCode::BAD_REQUEST)?;
    let jwt = cookie.value();
    let mut validation = Validation::default();
    validation.required_spec_claims.remove("exp");
    let data = decode(jwt, &DecodingKey::from_secret(b"asd"), &validation).unwrap();
    Ok(Json(data.claims))
}

async fn p3(token: String) -> Result<Json<Value>, StatusCode> {
    let key = include_bytes!("../day16_santa_public_key.pem");
    let header = decode_header(&token).map_err(|_| StatusCode::BAD_REQUEST)?;
    let alg = header.alg;
    let mut validation = Validation::default();
    validation.algorithms = vec![alg];
    validation.required_spec_claims.remove("exp");
    let key = DecodingKey::from_rsa_pem(key).unwrap();
    let p = decode(&token, &key, &validation);
    if let Err(err) = p {
        let ret = match err.into_kind() {
            ErrorKind::InvalidSignature => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };
        return Err(ret);
    }
    Ok(Json(p.unwrap().claims))
}

pub fn router() -> Router {
    Router::new()
        .route("/16/wrap", post(p1))
        .route("/16/unwrap", get(p2))
        .route("/16/decode", post(p3))
        .layer(CookieManagerLayer::new())
}
