use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use libsql_client::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInterestRequest {
    pub name: String,
    pub email: String,
    pub contact_no: String,
    pub uni_id: String,
    pub uni_name: String,
    pub where_you_reside: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub id: String,
    pub entity: String,
    pub amount: Number,
    pub amount_paid: Number,
    pub amount_due: Number,
    pub currency: String,
    pub receipt: String,
    pub offer_id: Option<String>,
    pub status: String,
    pub attempts: Number,
    pub notes: CreateOrderNotes,
    pub created_at: Number
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderTicketData {
    pub name: String,
    pub email: String,
    pub contact_no: String,
    pub uni_id: String,
    pub uni_name: String,
    pub where_you_reside: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequestUnParsed {
    pub ticket_type: String,
    pub ticket_data: CreateOrderTicketData
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub amount: Number,
    pub currency: String,
    pub receipt: String,
    pub notes: CreateOrderNotes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderNotes {
    pub notes_key_1: String,
    pub notes_key_2: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvStore {
    pub rpay_id: String,
    pub rpay_secret: String,
}

#[derive(Debug)]
pub struct StateStore {
    pub sql_client: Client,
    pub env_store: Arc<EnvStore>,
}