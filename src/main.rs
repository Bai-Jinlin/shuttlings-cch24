use axum::Router;
use sqlx::PgPool;

mod day0;
mod day12;
mod day16;
mod day19;
mod day2;
mod day5;
mod day9;

#[shuttle_runtime::main]
async fn main(
    // #[shuttle_shared_db::Postgres(local_uri = "postgres://shuttle:qwe@localhost:5432/app_test")]
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let d0 = day0::router();
    let d2 = day2::router();
    let d5 = day5::router();
    let d9 = day9::router();
    let d12 = day12::router();
    let d16 = day16::router();
    let d19 = day19::router(pool.clone());

    let router = Router::new()
        .merge(d0)
        .merge(d2)
        .merge(d5)
        .merge(d9)
        .merge(d12)
        .merge(d16)
        .merge(d19);
    Ok(router.into())
}