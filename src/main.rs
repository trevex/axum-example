use axum::{extract::State, http::StatusCode, routing::get, Router};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::net::SocketAddr;
use thiserror::Error;
use tokio_postgres::NoTls;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("postgres error")]
    PostgresError(#[from] tokio_postgres::Error),
    #[error("faild to build connection pool")]
    PoolBuildError(#[from] deadpool_postgres::BuildError),
    #[error("unknown database error")]
    Unknown,
}

#[tokio::main]
async fn main() {
    // TODO: return error rather than unwrapping
    let pool = create_pool("host=localhost user=postgres dbname=postgres").unwrap();
    let app = Router::new().route("/", get(test_pool)).with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_pool(conn_str: &str) -> Result<Pool, DatabaseError> {
    let pg_config = conn_str.parse::<tokio_postgres::Config>()?;
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::builder(mgr)
        .runtime(Runtime::Tokio1)
        .max_size(16)
        .build()?;
    Ok(pool)
}

// TODO: do something like https://github.com/uuip/axum-demo/blob/main/src/common/error.rs
async fn test_pool(State(pool): State<Pool>) -> Result<String, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let row = conn
        .query_one("select 1 + 1", &[])
        .await
        .map_err(internal_error)?;
    let two: i32 = row.try_get(0).map_err(internal_error)?;

    Ok(two.to_string())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
