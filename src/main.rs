use pay_n_ticketing_ee::{structs::*, services::*};

use std::sync::Arc;
use libsql_client::Client;
use axum::{
    routing::{get, post},
    Router
};
use shuttle_secrets::SecretStore;
use tower_http::cors::{CorsLayer, Any};

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_turso::Turso(addr = "{secrets.DB_TURSO_URI}", token = "{secrets.DB_TURSO_TOKEN}", local_addr = "{secrets.DB_TURSO_URI}")]
    sql_client: Client,
) -> shuttle_axum::ShuttleAxum {
    
    let sql_client = sql_client;
    let ev_state: Arc<EnvStore> = Arc::new(EnvStore {
        rpay_id: secret_store.get("RAZOR_PAY_KEY_ID").unwrap(),
        rpay_secret: secret_store.get("RAZOR_PAY_KEY_SECRET").unwrap(),
    });

    let state = Arc::new(StateStore {
        sql_client,
        env_store: ev_state,
    });

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/order", post(generate_order))
        .route("/interest", post(register_interest))
        .with_state(state)
        .layer(cors);

    Ok(router.into())
}
