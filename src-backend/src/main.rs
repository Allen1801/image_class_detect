mod model;
use crate::model::model_tasks::detect_image;

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
            Cors::default()
            .allowed_origin("https://imageclassydetect.vercel.app") // Allow your frontend
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Content-Type"])
            .supports_credentials(),
            )
            .wrap(middleware::Logger::default())
            .service(detect_image)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}


