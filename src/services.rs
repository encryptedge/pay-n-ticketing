use crate::{structs::*, mailer::send_ticket};
use axum::extract::{State, Json, Path, Query};
use reqwest::StatusCode;
use serde_json::Number;
use uuid::Uuid;
use chrono;
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose};
use libsql_client::{args, Statement};

pub async fn generate_order(
    State(state): State<Arc<StateStore>>,
    Json(payload): Json<CreateOrderRequestUnParsed>,
) -> Result<axum::Json<RazorPayOrderResponse>, StatusCode> {
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
            notes_key_2: payload.ticket_type.clone(),
            notes_key_3: "RCSCTF2024".to_string(),
            notes_key_4: None,
            notes_key_5: payload.ticket_data.name.clone(),
        }
    };

    if payload.ticket_type == "student_pass" {
        // order_payload.amount = Number::from_f64(25000.0).unwrap();
        return Err(StatusCode::BAD_REQUEST)
    } else if payload.ticket_type == "standard_pass" {
        // order_payload.amount = Number::from_f64(35000.0).unwrap();
        return Err(StatusCode::BAD_REQUEST)
    } else if payload.ticket_type == "professional_pass" {
        // order_payload.amount = Number::from_f64(50000.0).unwrap();
        return Err(StatusCode::BAD_REQUEST)
    } else if payload.ticket_type == "offline_pass" {
        // order_payload.amount = Number::from_f64(50000.0).unwrap();
        return Err(StatusCode::BAD_REQUEST)
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    // sql_client.execute(Statement::with_args(
    //     "INSERT INTO ticket (id, ticket_type, name, email, contact_no, uni_id, uni_name, where_you_reside, booked_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    //     args![id.clone(), payload.ticket_type.clone(), payload.ticket_data.name, payload.ticket_data.email, payload.ticket_data.contact_no, payload.ticket_data.uni_id, payload.ticket_data.uni_name, payload.ticket_data.where_you_reside, current_ts, current_ts])
    // ).await.unwrap();

    // let order_data: serde_json::Value = serde_json::json!({
    //     "amount": order_payload.amount,
    //     "currency": order_payload.currency,
    //     "receipt": order_payload.receipt,
    //     "notes": {
    //         "notes_key_1": order_payload.notes.notes_key_1,
    //         "notes_key_2": order_payload.notes.notes_key_2,
    //         "notes_key_3": order_payload.notes.notes_key_3,
    //         "notes_key_5": order_payload.notes.notes_key_5,
    //     }
    // });

    // let request = client.request(reqwest::Method::POST, "https://api.razorpay.com/v1/orders")
    //     .headers(headers)
    //     .json(&order_data);

    // let response = request.send().await.unwrap();
    // let body = response.text().await.unwrap();

    // let order: RazorPayOrderResponse = serde_json::from_str(&body).unwrap();

    // sql_client.execute(
    //     Statement::with_args("UPDATE ticket set order_id = ? WHERE id = ?", args![order.id.clone(), id])
    // ).await.unwrap();

    // Ok(axum::Json(order))
}

pub async fn check_payments(
    State(state): State<Arc<StateStore>>,
    Path(order_id): Path<String>,
) -> String {
    let current_date = chrono::Local::now();
    let current_ts = current_date.timestamp();
    let ticket_id = format!("RCSCTF2024T{}", current_ts.to_string());
    let rpay_key_id = &state.env_store.rpay_id;
    let rpay_key_secret = &state.env_store.rpay_secret;

    let pre_key = format!("{}:{}", rpay_key_id, rpay_key_secret);
    let encoded_key = general_purpose::STANDARD_NO_PAD.encode(pre_key.as_bytes());

    let client = reqwest::Client::builder()
        .build().unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", format!("Basic {}", encoded_key).parse().unwrap());

    let request = client.request(reqwest::Method::GET, format!("https://api.razorpay.com/v1/orders/{}", order_id))
        .headers(headers.clone());

    let response = request.send().await.unwrap();
    if response.status() != StatusCode::OK {
        return format!("FAIL for Razorpay Fetch!");
        // return response.text().await.unwrap();
    }

    let body = response.text().await.unwrap();
    let order: RazorPayOrderResponse = serde_json::from_str(&body).unwrap();

    if order.status == "paid" {
        let sql_client = &state.sql_client;
        sql_client.execute(
            Statement::with_args("UPDATE ticket set is_paid = true, ticket_id = ? WHERE id = ?", args![ticket_id.clone(), order.notes.notes_key_1.clone()])
        ).await.unwrap();
        let ticket = sql_client.execute(
            Statement::with_args("SELECT email, name, ticket_type, id  FROM ticket WHERE id = ?", args![order.notes.notes_key_1])
        ).await.unwrap();
        let ticket = ticket.rows;
        let ticket = ticket[0].clone();
        let ticket: CreateTicketMailingRequest = CreateTicketMailingRequest {
            payee_email: ticket.values[0].clone().to_string(),
            payee_name: ticket.values[1].clone().to_string(),
            payee_ticket_id: ticket_id.clone(),
            ticket_type: ticket.values[2].clone().to_string(),
            id: ticket.values[3].clone().to_string(),
        };
        let mailer_auth = MailerAuth {
            username: state.env_store.mailer_username.clone(),
            password: state.env_store.mailer_password.clone(),
            mailer_url: state.env_store.mailer_url.clone(),
        };
        let updated_notes = CreateOrderNotes {
            notes_key_1: ticket.id.clone(),
            notes_key_2: ticket.ticket_type.clone(),
            notes_key_3: "RCSCTF2024".to_string(),
            notes_key_4: Some(ticket_id.clone()),
            notes_key_5: ticket.payee_name.clone(),
        };

        let updated_notes = RazorpayPaymentNotesUpdate {
            notes: updated_notes,
        };

        let request = client.request(reqwest::Method::PATCH, format!("https://api.razorpay.com/v1/orders/{}", order.id))
            .headers(headers)
            .json(&updated_notes);

        let response = request.send().await.unwrap();
        if response.status() != StatusCode::OK {
            // return format!("FAIL for Update Razorpay!");
            return response.text().await.unwrap();
        }

        let mailer = send_ticket(ticket, mailer_auth).await;
        return mailer
    } else {
        return format!("FAIL, order not paid!!")
    }
}

pub async fn register_interest(
    State(state): State<Arc<StateStore>>,
    Json(payload): Json<CreateInterestRequest>,
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

pub async fn fetch_all_paid_tickets(
    State(state): State<Arc<StateStore>>,
    Query(auth): Query<PageAuth>,
) -> Result<axum::Json<Vec<PaidTicket>>, StatusCode> {
    if auth.key != state.env_store.fetch_token {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let sql_client = &state.sql_client;
    let tickets = sql_client.execute(
        Statement::with_args("SELECT id, ticket_type, name, email, contact_no, uni_id, uni_name, where_you_reside, ticket_id, is_paid  FROM ticket WHERE is_paid = ?", args![1])
    ).await.unwrap();
    let tickets = tickets.rows;
    let mut tickets_vec: Vec<PaidTicket> = Vec::new();
    for ticket in tickets {
        let ticket = PaidTicket {
            id: ticket.values[0].clone().to_string(),
            ticket_type: ticket.values[1].clone().to_string(),
            name: ticket.values[2].clone().to_string(),
            email: ticket.values[3].clone().to_string(),
            contact_no: ticket.values[4].clone().to_string(),
            uni_id: ticket.values[5].clone().to_string(),
            uni_name: ticket.values[6].clone().to_string(),
            where_you_reside: ticket.values[7].clone().to_string(),
            ticket_id: ticket.values[8].clone().to_string(),
            is_paid: ticket.values[9].clone().to_string(),
        };
        tickets_vec.push(ticket);
    }
    Ok(axum::Json(tickets_vec))
}

pub async fn fetch_all_unpaid_tickets(
    State(state): State<Arc<StateStore>>,
    Query(auth): Query<PageAuth>,
) -> Result<axum::Json<Vec<PaidTicket>>, StatusCode> {
    if auth.key != state.env_store.fetch_token {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let sql_client = &state.sql_client;
    let tickets = sql_client.execute(
        "SELECT id, ticket_type, name, email, contact_no, uni_id, uni_name, where_you_reside, ticket_id, is_paid  FROM ticket WHERE is_paid IS NULL"
    ).await.unwrap();
    let tickets = tickets.rows;
    let mut tickets_vec: Vec<PaidTicket> = Vec::new();
    for ticket in tickets {
        let ticket = PaidTicket {
            id: ticket.values[0].clone().to_string(),
            ticket_type: ticket.values[1].clone().to_string(),
            name: ticket.values[2].clone().to_string(),
            email: ticket.values[3].clone().to_string(),
            contact_no: ticket.values[4].clone().to_string(),
            uni_id: ticket.values[5].clone().to_string(),
            uni_name: ticket.values[6].clone().to_string(),
            where_you_reside: ticket.values[7].clone().to_string(),
            ticket_id: ticket.values[8].clone().to_string(),
            is_paid: ticket.values[9].clone().to_string(),
        };
        tickets_vec.push(ticket);
    }
    Ok(axum::Json(tickets_vec))
}