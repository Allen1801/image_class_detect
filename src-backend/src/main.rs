mod model;
use crate::model::model_tasks::detect_image;

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
            Cors::default()
                .allow_any_origin() // Allows all origins. For production, specify allowed origins.
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::CONTENT_TYPE])
                .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .service(web::scope("/detect_image").service(detect_image))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}


