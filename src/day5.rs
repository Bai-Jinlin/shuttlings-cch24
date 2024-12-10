
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use cargo_manifest::Manifest;
use serde::Deserialize;

#[derive(Debug)]
struct Metadata {
    orders: Vec<Orders>,
}
#[derive(Debug, Deserialize)]
struct Orders {
    item: String,
    quantity: u32,
}

#[derive(Debug)]
enum MyErr {
    MetadataError,
    InvalidError,
    MagicError,
    ContentUnsupportedError,
    UnknownError,    
}
impl IntoResponse for MyErr {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InvalidError => {
                (StatusCode::BAD_REQUEST, "Invalid manifest".to_string()).into_response()
            }
            Self::MetadataError => StatusCode::NO_CONTENT.into_response(),
            Self::MagicError => {
                (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response()
            }
            Self::ContentUnsupportedError => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
            Self::UnknownError => panic!(),
        }
    }
}

enum ContentType {
    Toml,
    Yaml,
    Json
}

#[async_trait]
impl<S> FromRequest<S> for Metadata
where
    S: Send + Sync,
{
    type Rejection = MyErr;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(content_type) = check_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state)
                .await
                .map_err(|_| MyErr::UnknownError)?;

            let manifest = match content_type {
                ContentType::Toml => Manifest::from_slice(&bytes).map_err(|_| MyErr::InvalidError)?,
                ContentType::Yaml => serde_yaml::from_slice(&bytes).map_err(|_| MyErr::InvalidError)?,
                ContentType::Json => serde_json::from_slice(&bytes).map_err(|_|MyErr::InvalidError)?,
            };
                

            let package = manifest.package;
            if let Some(package) = &package {
                if let Some(keywords) = &package.keywords {
                    if let Some(keywords) = keywords.clone().as_local() {
                        if !keywords.contains(&"Christmas 2024".to_string()) {
                            return Err(MyErr::MagicError);
                        }
                    }
                }else{
                    return Err(MyErr::MagicError);
                }
            }

            let metadata = package.map(|p| p.metadata).flatten();

            let metadata = if let Some(v) = metadata {
                let mut orders = Vec::new();
                if let Some(m) = v.as_table() {
                    let v = m.get("orders");
                    if let Some(a) = v {
                        for v in a.as_array().unwrap() {
                            let m: Result<Orders, toml::de::Error> = v.clone().try_into();
                            if let Ok(m) = m {
                                orders.push(m);
                            }
                        }
                    }
                }
                if orders.is_empty() {
                    return Err(MyErr::MetadataError);
                }
                Metadata { orders }
            } else {
                return Err(MyErr::MetadataError);
            };

            Ok(metadata)
        } else {
            Err(MyErr::ContentUnsupportedError)
        }
    }
}

fn check_content_type(headers: &HeaderMap) -> Option<ContentType> {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return None;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return None;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return None;
    };
    if mime.type_() == "application" {
        if mime.subtype() == "toml" || mime.suffix().map_or(false, |name| name == "toml") {
            return Some(ContentType::Toml);
        } else if mime.subtype() == "yaml" || mime.suffix().map_or(false, |name| name == "yaml") {
            return Some(ContentType::Yaml);
        } else if mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json") {
            return Some(ContentType::Json);
        }
    }
    None
}

async fn p1(m: Metadata) -> String {
    let mut ret = String::new();
    for s in m
        .orders
        .into_iter()
        .map(|o| format!("{}: {}", o.item, o.quantity))
    {
        ret.push_str(&s);
        ret.push('\n');
    }
    if !ret.is_empty() {
        ret.pop();
    }
    ret
}

pub fn router() -> Router {
    Router::new().route("/5/manifest", post(p1))
}