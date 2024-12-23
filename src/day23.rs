use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tera::escape_html;
use toml::{map::Map, Value};
use tower_http::services::ServeDir;

async fn star() -> Html<&'static str> {
    let html = r#"<div id="star" class="lit"></div>"#;

    Html(html)
}

async fn present(Path(color): Path<String>) -> Result<Html<String>, StatusCode> {
    let color = escape_html(&color);
    let next_color = match &*color {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return Err(StatusCode::IM_A_TEAPOT),
    };

    let html = format!(
        r#"
        <div class="present {color}" hx-get="/23/present/{next_color}" hx-swap="outerHTML">
            <div class="ribbon"></div>
            <div class="ribbon"></div>
            <div class="ribbon"></div>
            <div class="ribbon"></div>
        </div>
        "#,
    );

    Ok(Html(html))
}

async fn ornament(Path((state, n)): Path<(String, String)>) -> Result<Html<String>, StatusCode> {
    let state = escape_html(&state);
    let n = escape_html(&n);

    let next_state = match &*state {
        "on" => "off",
        "off" => "on",
        _ => return Err(StatusCode::IM_A_TEAPOT),
    };

    let mut class = "ornament".to_string();
    if state == "on" {
        class.push(' ');
        class.push_str("on");
    }

    let html = format!(
        r#"
        <div class="{class}" id="ornament{n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{next_state}/{n}" hx-swap="outerHTML">
        </div>
        "#,
    );

    Ok(Html(html))
}

#[derive(Deserialize)]
struct Package {
    _name: Option<String>,
    _source: Option<String>,
    _version: Option<String>,
    checksum: String,
}
impl Package {
    fn cal(self) -> Option<(String, i32, i32)> {
        let checksum = self.checksum;
        let color = checksum.get(0..6)?;
        color
            .chars()
            .all(|c| match c {
                '0'..='9' => true,
                'a'..='f' => true,
                _ => false,
            })
            .then(|| ())?;
        let top = checksum.get(6..8)?;
        let left = checksum.get(8..10)?;
        let top = i32::from_str_radix(top, 16).ok()?;
        let left = i32::from_str_radix(left, 16).ok()?;

        Some((format!("#{color}"), top, left))
    }
}

async fn lockfile(mut multipart: Multipart) -> Result<Html<String>, StatusCode> {
    let mut htmls = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let data = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        let payload: Map<String, Value> =
            toml::from_str(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
        let packages = payload["package"].as_array().unwrap();

        for package in packages {
            if let Ok(payload) = package.clone().try_into::<Package>() {
                let d = payload.cal().ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
                htmls.push(d);
            }
        }
    }
    if htmls.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let html = htmls
        .into_iter()
        .map(|(color, top, left)| {
            format!(r#"<div style="background-color:{color};top:{top}px;left:{left}px;"></div>"#)
        })
        .collect();

    Ok(Html(html))
}

pub fn router() -> Router {
    Router::new()
        .route("/23/star", get(star))
        .route("/23/present/:color", get(present))
        .route("/23/ornament/:state/:n", get(ornament))
        .route("/23/lockfile", post(lockfile))
        .nest_service("/assets", ServeDir::new("assets"))
}
