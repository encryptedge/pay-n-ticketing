use reqwest::StatusCode;
use uuid::Uuid;
use chrono;
use libsql_client::{args, Client, Statement};
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose};
use axum::{
    extract::{self, State},
    routing::{get, post},
    Router
};
use serde_json::Number;
use serde::{Deserialize, Serialize};
use shuttle_secrets::SecretStore;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInterestRequest {
    name: String,
    email: String,
    contact_no: String,
    uni_id: String,
    uni_name: String,
    where_you_reside: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    id: String,
    entity: String,
    amount: Number,
    amount_paid: Number,
    amount_due: Number,
    currency: String,
    receipt: String,
    offer_id: Option<String>,
    status: String,
    attempts: Number,
    notes: CreateOrderNotes,
    created_at: Number
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderTicketData {
    name: String,
    email: String,
    contact_no: String,
    uni_id: String,
    uni_name: String,
    where_you_reside: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequestUnParsed {
    ticket_type: String,
    ticket_data: CreateOrderTicketData
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    amount: Number,
    currency: String,
    receipt: String,
    notes: CreateOrderNotes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderNotes {
    notes_key_1: String,
    notes_key_2: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvStore {
    rpay_id: String,
    rpay_secret: String,
}

#[derive(Debug)]
pub struct StateStore {
    pub sql_client: Client,
    pub env_store: Arc<EnvStore>,
}

async fn generate_order(
    State(state): State<Arc<StateStore>>,
    extract::Json(payload): extract::Json<CreateOrderRequestUnParsed>,
) -> Result<axum::Json<CreateOrderResponse>, StatusCode> {
    let rpay_key_id = &state.env_store.rpay_id;
    let rpay_key_secret = &state.env_store.rpay_secret;

    let pre_key = format!("{}:{}", rpay_key_id, rpay_key_secret);
    let encoded_key = general_purpose::STANDARD_NO_PAD.encode(pre_key.as_bytes());
    let client = reqwest::Client::builder()
        .build().unwrap();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", format!("Basic {}", encoded_key).parse().unwrap());

    let sql_client = &state.sql_client;
    let id = Uuid::new_v4();
    let id = id.to_string();
    let current_date = chrono::Local::now();
    let current_ts = current_date.timestamp();

    let mut order_payload = CreateOrderRequest {
        amount: Number::from_f64(0.0).unwrap(),
        currency: "INR".to_string(),
        receipt: "EECTF".to_string(),
        notes: CreateOrderNotes {
            notes_key_1: id.clone(),
            notes_key_2: payload.ticket_type.clone()
        }
    };

    if payload.ticket_type == "student_pass" {
        order_payload.amount = Number::from_f64(10000.0).unwrap();
    } else if payload.ticket_type == "standard_pass" {
        order_payload.amount = Number::from_f64(20000.0).unwrap();
    } else if payload.ticket_type == "professional_pass" {
        order_payload.amount = Number::from_f64(30000.0).unwrap();
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    sql_client.execute(Statement::with_args(
        "INSERT INTO ticket (id, ticket_type, name, email, contact_no, uni_id, uni_name, where_you_reside, booked_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        args![id.clone(), payload.ticket_type.clone(), payload.ticket_data.name, payload.ticket_data.email, payload.ticket_data.contact_no, payload.ticket_data.uni_id, payload.ticket_data.uni_name, payload.ticket_data.where_you_reside, current_ts, current_ts])
    ).await.unwrap();

    let order_data: serde_json::Value = serde_json::json!({
        "amount": order_payload.amount,
        "currency": order_payload.currency,
        "receipt": order_payload.receipt,
        "notes": {
            "notes_key_1": order_payload.notes.notes_key_1,
            "notes_key_2": order_payload.notes.notes_key_2
        }
    });

    let request = client.request(reqwest::Method::POST, "https://api.razorpay.com/v1/orders")
        .headers(headers)
        .json(&order_data);

    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();

    let order: CreateOrderResponse = serde_json::from_str(&body).unwrap();

    Ok(axum::Json(order))
}

async fn register_interest(
    State(state): State<Arc<StateStore>>,
    extract::Json(payload): extract::Json<CreateInterestRequest>,
) -> &'static str {
    let sql_client = &state.sql_client;
    let id = Uuid::new_v4();
    let id = id.to_string();
    let current_date = chrono::Local::now();
    let current_ts = current_date.timestamp();
    sql_client.execute(Statement::with_args(
        "INSERT INTO interest (id, name, email, contact_no, uni_id, uni_name, where_you_reside, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        args![id, payload.name, payload.email, payload.contact_no, payload.uni_id, payload.uni_name, payload.where_you_reside, current_ts])
    ).await.unwrap();
    "OK!"
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_turso::Turso(addr = "{secrets.DB_TURSO_URI}", token = "{secrets.DB_TURSO_TOKEN}", local_addr = "{secrets.DB_TURSO_URI}")]
    sql_client: Client,
) -> shuttle_axum::ShuttleAxum {
    sql_client.execute("CREATE TABLE IF NOT EXISTS interest (id TEXT PRIMARY KEY, name TEXT, email TEXT, contact_no TEXT, uni_id TEXT, uni_name TEXT, where_you_reside TEXT, created_at TEXT)").await.unwrap();

    let sql_client = sql_client;
    let ev_state: Arc<EnvStore> = Arc::new(EnvStore {
        rpay_id: secret_store.get("RAZOR_PAY_KEY_ID").unwrap(),
        rpay_secret: secret_store.get("RAZOR_PAY_KEY_SECRET").unwrap(),
    });

    let state = Arc::new(StateStore {
        sql_client,
        env_store: ev_state,
    });

    let router = Router::new()
        .route("/hello", get(hello_world))
        .route("/order", post(generate_order))
        .route("/interest", post(register_interest))
        .with_state(state);

    Ok(router.into())
}
