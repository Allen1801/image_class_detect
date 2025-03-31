mod model;
use crate::model::model_tasks::detect_image;

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
            Cors::permissive()
                .allow_any_origin() // Allows all origins. For production, specify allowed origins.
                .allow_any_method()
                .allow_any_header()
                .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .service(detect_image)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}


