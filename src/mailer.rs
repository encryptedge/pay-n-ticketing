use base64::{Engine as _, engine::general_purpose};

use crate::structs::{CreateTicketMailingRequest, MailerAuth};

pub async fn send_ticket(ticket_request: CreateTicketMailingRequest, auth_creds: MailerAuth) -> String {
    let client = reqwest::Client::builder()
        .build().unwrap();
    let auth_string = format!("{}:{}", auth_creds.username, auth_creds.password);
    let encoded_key = general_purpose::STANDARD_NO_PAD.encode(auth_string.as_bytes());

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Authorization", format!("Basic {}", encoded_key).parse().unwrap());

    let request = client.request(reqwest::Method::POST, auth_creds.mailer_url)
        .headers(headers)
        .json(&ticket_request);

    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();

    format!("{}", body)
}