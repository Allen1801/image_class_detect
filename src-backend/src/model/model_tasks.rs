use base64::{engine::general_purpose, Engine};
use crate::{model::{constants::{CONF_THRESHOLD, IOU_THRESHOLD}, model_functions::functions::model_run}};

/* <--- THIS IS FOR INVOKE CALL FOR FRONTEND ---> */


pub fn ocr_image(img: &str) -> Vec<String> {
    // initialize weights
    const WEIGHTS_SAFETENSORS: &[u8] = include_bytes!("pretrained/cheque.safetensors");
    // Convert image into base64
    let split_image = img.split(',').last().unwrap_or("");
    let image_base64 = general_purpose::STANDARD.decode(split_image).expect("Failed to decode image");
    // Initialize model and run
    let model = model_run(WEIGHTS_SAFETENSORS);


    let bbox = match model.run_img_ocr(image_base64, CONF_THRESHOLD, IOU_THRESHOLD) {
        Ok(label) => label, // Directly return the label
        Err(err) => format!("{}", err), // Return the error as a string
    };


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
