use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewsletterRequest {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    text: String,
    html: String,
}

pub async fn publish_newsletter(req: web::Json<NewsletterRequest>) -> HttpResponse {
    HttpResponse::Ok().finish()
}