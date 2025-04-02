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
                .allowed_origin("https://imageclassydetect.vercel.app")// Allows all origins. For production, specify allowed origins.
                .allow_any_method()
                .allow_any_header()
                .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .service(detect_image)
    })
    .bind("https://image-class-detect.onrender.com")?
    .run()
    .await
}


