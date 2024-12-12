use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize)]
pub struct SubscriptionRequest {
    username: String,
    email: String,
}

impl SubscriptionRequest {
    pub fn new(username: String, email: String) -> Self {
        SubscriptionRequest { username, email }
    }
}



async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

async fn health_check(_req: HttpRequest) -> impl Responder {
    let body = "{\"metric1\":1000, \"metric2\":2000}";
    HttpResponse::Ok().json(body)
}
async fn subscribe(info: web::Json<SubscriptionRequest>) -> HttpResponse {
    println!("{}",info.username);
    println!("{}",info.email);
    if info.username == ""{
        return HttpResponse::BadRequest().body("username is missing")
    }
    if info.email == ""{
        return HttpResponse::BadRequest().body("email is missing")
    }

    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
