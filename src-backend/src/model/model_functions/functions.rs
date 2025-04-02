use crate::model::classes::YOLO_CLASSES;
use crate::model::model_structs::structs::{ Model, ModelData, Bbox };
use candle_core::{ IndexOp, Result, Tensor, DType, D };
use image::DynamicImage;
use serde_json::json;

/* 
    <--- ADD FUNCTIONS BASED ON YOUR NEEDS START HERE ---> 
    
    IMPORTANT:
        don't make modification code after the tail of this code block
*/

use image::Rgba;
use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};

use ab_glyph::{FontRef, PxScale};
use imageproc::rect::Rect;
use base64::{engine::general_purpose, Engine};
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use std::io::Cursor;

pub fn report_detect(
    pred: &Tensor,
    img: DynamicImage,
    w: usize,
    h: usize,
    conf_threshold: f32,
    iou_threshold: f32,
) -> Result<serde_json::Value> {
    let (pred_size, npreds) = pred.dims2()?;
    let nclasses = pred_size - 4;
    let conf_threshold = conf_threshold.clamp(0.0, 1.0);
    let iou_threshold = iou_threshold.clamp(0.0, 1.0);

    let mut bboxes: Vec<Vec<Bbox>> = (0..nclasses).map(|_| vec![]).collect();
    let mut labels = Vec::new();

    for index in 0..npreds {
        let pred = Vec::<f32>::try_from(pred.i((.., index))?)?;
        let confidence = *pred[4..].iter().max_by(|x, y| x.total_cmp(y)).unwrap();
        if confidence > conf_threshold {
            let mut class_index = 0;
            for i in 0..nclasses {
                if pred[4 + i] > pred[4 + class_index] {
                    class_index = i
                }
            }
            if pred[class_index + 4] > 0. {
                let bbox = Bbox {
                    xmin: pred[0] - pred[2] / 2.,
                    ymin: pred[1] - pred[3] / 2.,
                    xmax: pred[0] + pred[2] / 2.,
                    ymax: pred[1] + pred[3] / 2.,
                    confidence,
                    keypoints: vec![],
                };
                bboxes[class_index].push(bbox);
                labels.push(YOLO_CLASSES[class_index].to_string());
            }
        }
    }

    non_maximum_suppression(&mut bboxes, iou_threshold);

    let (initial_h, initial_w) = (img.height() as f32, img.width() as f32);
    let w_ratio = initial_w / w as f32;
    let h_ratio = initial_h / h as f32;

    let mut img = img.to_rgba8(); // Convert to mutable RGBA image

    // Load font (replace with an actual font file path)
    let font_data = include_bytes!(r"../../assets/arial.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Failed to load font");

    let scale = PxScale::from(20.0); // Fix: Convert to PxScale

    for (class_index, bboxes_for_class) in bboxes.iter_mut().enumerate() {
        let label = YOLO_CLASSES[class_index].to_string();
        let color = Rgba([255, 0, 0, 255]); // Red bounding box
        println!("Class {}: {}", class_index, label);
        println!("Bboxes: {:?}", bboxes_for_class);
        for b in bboxes_for_class.iter_mut() {
            b.xmin = (b.xmin * w_ratio).clamp(0., initial_w - 1.);
            b.ymin = (b.ymin * h_ratio).clamp(0., initial_h - 1.);
            b.xmax = (b.xmax * w_ratio).clamp(0., initial_w - 1.);
            b.ymax = (b.ymax * h_ratio).clamp(0., initial_h - 1.);

            let rect = Rect::at(b.xmin as i32, b.ymin as i32)
                .of_size((b.xmax - b.xmin) as u32, (b.ymax - b.ymin) as u32);
            
            draw_hollow_rect_mut(&mut img, rect, color);

            let text_pos = (b.xmin as i32, (b.ymin - 10.0).max(0.0) as i32);
            draw_text_mut(&mut img, Rgba([255, 255, 255, 255]), text_pos.0, text_pos.1, scale, &font, &label);
        }
    }

    // Convert image to PNG and encode as Base64
    let mut buffer = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buffer);
    encoder
        .write_image(&img, img.width(), img.height(), image::ExtendedColorType::Rgba8)
        .expect("Failed to encode image as PNG");

    let base64_string = general_purpose::STANDARD.encode(&buffer.into_inner());
    Ok(json!({
        "image": base64_string,
        "labels": labels
    }))
}



// pub fn report_classified(pred: &Tensor, conf_threshold: f32) -> Result<String> {
//     let (_, npreds) = pred.dims2()?;
//     let conf_threshold = conf_threshold.clamp(0.0, 1.0);
    
//     let mut detected_classes = Vec::new();
    
//     for index in 0..npreds {
//         let pred = Vec::<f32>::try_from(pred.i((.., index))?)?;
//         let confidence = *pred[4..].iter().max_by(|x, y| x.total_cmp(y)).unwrap();
//         if confidence > conf_threshold {
//             let class_index = (4..pred.len()).max_by(|&i, &j| pred[i].total_cmp(&pred[j])).unwrap() - 4;
//             let class_name = NAMES[class_index].to_string();
//             if !detected_classes.contains(&class_name) {
//                 detected_classes.push(class_name);
//             }
//         }
//     }
    
//     Ok(detected_classes.join(", "))
// }

/* <--- ADD FUNCTIONS BASED ON YOUR NEEDS ENDS HERE ---> */

pub fn model_run(weights: &[u8]) -> Model {
    // initialize model data
    let model_data = ModelData {
        weights: Vec::from(weights),
        model_size: "n".to_string()
    };
    // load model
    let model = Model::load(model_data).expect("failed to load model");
    
    return model
}

fn non_maximum_suppression(bboxes: &mut [Vec<Bbox>], threshold: f32) {
    // Perform non-maximum suppression.
    for bboxes_for_class in bboxes.iter_mut() {
        bboxes_for_class.sort_by(|b1, b2| b2.confidence.partial_cmp(&b1.confidence).unwrap());
        let mut current_index = 0;
        for index in 0..bboxes_for_class.len() {
            let mut drop = false;
            for prev_index in 0..current_index {
                let iou = iou(&bboxes_for_class[prev_index], &bboxes_for_class[index]);
                if iou > threshold {
                    drop = true;
                    break;
                }
            }
            if !drop {
                bboxes_for_class.swap(current_index, index);
                current_index += 1;
            }
        }
        bboxes_for_class.truncate(current_index);
    }
}

// Intersection over union of two bounding boxes.
fn iou(b1: &Bbox, b2: &Bbox) -> f32 {
    let b1_area = (b1.xmax - b1.xmin + 1.) * (b1.ymax - b1.ymin + 1.);
    let b2_area = (b2.xmax - b2.xmin + 1.) * (b2.ymax - b2.ymin + 1.);
    let i_xmin = b1.xmin.max(b2.xmin);
    let i_xmax = b1.xmax.min(b2.xmax);
    let i_ymin = b1.ymin.max(b2.ymin);
    let i_ymax = b1.ymax.min(b2.ymax);
    let i_area = (i_xmax - i_xmin + 1.).max(0.) * (i_ymax - i_ymin + 1.).max(0.);
    i_area / (b1_area + b2_area - i_area)
}

pub fn dist2bbox(distance: &Tensor, anchor_points: &Tensor) -> Result<Tensor> {
    let chunks = distance.chunk(2, 1)?;
    let lt = &chunks[0];
    let rb = &chunks[1];
    let x1y1 = anchor_points.sub(lt)?;
    let x2y2 = anchor_points.add(rb)?;
    let c_xy = ((&x1y1 + &x2y2)? * 0.5)?;
    let wh = (&x2y2 - &x1y1)?;
    Tensor::cat(&[c_xy, wh], 1)
}

pub fn make_anchors(
    xs0: &Tensor,
    xs1: &Tensor,
    xs2: &Tensor,
    (s0, s1, s2): (usize, usize, usize),
    grid_cell_offset: f64,
) -> Result<(Tensor, Tensor)> {
    let dev = xs0.device();
    let mut anchor_points = vec![];
    let mut stride_tensor = vec![];
    for (xs, stride) in [(xs0, s0), (xs1, s1), (xs2, s2)] {
        // xs is only used to extract the h and w dimensions.
        let (_, _, h, w) = xs.dims4()?;
        let sx = (Tensor::arange(0, w as u32, dev)?.to_dtype(DType::F32)? + grid_cell_offset)?;
        let sy = (Tensor::arange(0, h as u32, dev)?.to_dtype(DType::F32)? + grid_cell_offset)?;
        let sx = sx
            .reshape((1, sx.elem_count()))?
            .repeat((h, 1))?
            .flatten_all()?;
        let sy = sy
            .reshape((sy.elem_count(), 1))?
            .repeat((1, w))?
            .flatten_all()?;
        anchor_points.push(Tensor::stack(&[&sx, &sy], D::Minus1)?);
        stride_tensor.push((Tensor::ones(h * w, DType::F32, dev)? * stride as f64)?);
    }
    let anchor_points = Tensor::cat(anchor_points.as_slice(), 0)?;
    let stride_tensor = Tensor::cat(stride_tensor.as_slice(), 0)?.unsqueeze(1)?;
    Ok((anchor_points, stride_tensor))
}
