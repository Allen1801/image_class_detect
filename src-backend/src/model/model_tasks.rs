use base64::{engine::general_purpose, Engine};
use actix_web::{post, web, HttpResponse, Error};
use crate::model::{constants::{CONF_THRESHOLD, IOU_THRESHOLD}, model_functions::functions::model_run};
use crate::model::model_structs::structs::ImageRequest;

/* <--- THIS IS FOR INVOKE CALL FOR FRONTEND ---> */



// Handler function
#[post("/detect_image")]
pub async fn detect_image(img: web::Json<ImageRequest>) -> Result<HttpResponse, Error> {
    // Initialize weights
    println!("Detecting image...");

    let img_base64 = img.image.clone();

    const WEIGHTS_SAFETENSORS: &[u8] = include_bytes!("pretrained/yolov8n.safetensors");

    // Convert image into base64
    let split_image = &img_base64.split(',').last().unwrap_or("");
    let image_base64 = general_purpose::STANDARD
        .decode(split_image)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid base64 image"))?;

    // Initialize model
    let model = model_run(WEIGHTS_SAFETENSORS);

    // Run OCR (assuming this returns `Result<String, candle_core::Error>`)
    let bbox = model.run_img_detect(image_base64, CONF_THRESHOLD, IOU_THRESHOLD)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("OCR failed: {}", e.to_string())))?;

    print!("{}", bbox);
    println!("Detection completed.");
    // Return response as JSON
    Ok(HttpResponse::Ok().json(bbox))
}


// pub fn classify_image(img: &str) -> String {
//     // initialize weights
//     const WEIGHTS_SAFETENSORS: &[u8] = include_bytes!("pretrained/yolov8.safetensors");
//     // Convert image into base64
//     let split_image = img.split(',').last().unwrap_or("");
//     let image_base64 = general_purpose::STANDARD.decode(split_image).expect("Failed to decode image");
//     // Initialize model and run
//     let model = model_run(WEIGHTS_SAFETENSORS);
//     // Return result
//     match model.run_img_classification(image_base64, CONF_THRESHOLD) {
//         Ok(label) => return label, // Directly return the label
//         Err(err) => return format!("{}", err), // Return the error as a string
//     }
// }
