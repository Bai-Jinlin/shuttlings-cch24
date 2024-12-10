use axum::Router;

mod day0;
mod day2;
mod day5;
mod day9;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {

    let d0 = day0::router();
    let d2 = day2::router();
    let d5 = day5::router();
    let d9 = day9::router();

    let router = Router::new().merge(d0).merge(d2).merge(d5).merge(d9);
    Ok(router.into())
}
