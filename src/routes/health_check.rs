use actix_web::{HttpRequest, HttpResponse, Responder};


pub async fn health_check(_req: HttpRequest) -> impl Responder {
    let body = "{\"metric1\":1000, \"metric2\":2000}";
    HttpResponse::Ok().json(body)
}



pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}