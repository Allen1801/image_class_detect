use serde::{ Serialize, Deserialize };
use candle_core::Tensor;
use candle_nn::{
    BatchNorm, Conv2d
};

/* <--- MODEL STRUCTURE AND METHODS FOR LOADING AND RUNNING INFERENCE ON THE STATIC IMAGE ---> */
#[derive(Serialize, Deserialize)]
pub struct ImageRequest {
    pub image: String, // Matches { "image": "data:image/png;base64,..." }
}

#[derive(Serialize, Deserialize)]
pub struct ModelData {
    pub weights: Vec<u8>,
    pub model_size: String,
}

pub struct Model {
    pub model: YoloV8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Multiples {
    pub depth: f64,
    pub width: f64,
    pub ratio: f64,
}

#[derive(Debug)]
pub struct Upsample {
    pub scale_factor: usize,
}

#[derive(Debug)]
pub struct ConvBlock {
    pub conv: Conv2d,
    pub bn: BatchNorm,
}

#[derive(Debug)]
pub struct Bottleneck {
    pub cv1: ConvBlock,
    pub cv2: ConvBlock,
    pub residual: bool,
}

#[derive(Debug)]
pub struct C2f {
    pub cv1: ConvBlock,
    pub cv2: ConvBlock,
    pub bottleneck: Vec<Bottleneck>,
}

#[derive(Debug)]
pub struct Sppf {
    pub cv1: ConvBlock,
    pub cv2: ConvBlock,
    pub k: usize,
}

#[derive(Debug)]
pub struct Dfl {
    pub conv: Conv2d,
    pub num_classes: usize,
}

#[derive(Debug)]
pub struct DarkNet {
    pub b1_0: ConvBlock,
    pub b1_1: ConvBlock,
    pub b2_0: C2f,
    pub b2_1: ConvBlock,
    pub b2_2: C2f,
    pub b3_0: ConvBlock,
    pub b3_1: C2f,
    pub b4_0: ConvBlock,
    pub b4_1: C2f,
    pub b5: Sppf,
}

#[derive(Debug)]
pub struct YoloV8Neck {
    pub up: Upsample,
    pub n1: C2f,
    pub n2: C2f,
    pub n3: ConvBlock,
    pub n4: C2f,
    pub n5: ConvBlock,
    pub n6: C2f,
}

#[derive(Debug)]
pub struct DetectionHead {
    pub dfl: Dfl,
    pub cv2: [(ConvBlock, ConvBlock, Conv2d); 3],
    pub cv3: [(ConvBlock, ConvBlock, Conv2d); 3],
    pub ch: usize,
    pub no: usize,
}

#[allow(dead_code)]
pub struct DetectionHeadOut {
    pub pred: Tensor,
    pub anchors: Tensor,
    pub strides: Tensor,
}

#[derive(Debug)]
pub struct YoloV8 {
    pub net: DarkNet,
    pub fpn: YoloV8Neck,
    pub head: DetectionHead,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct KeyPoint {
    pub x: f32,
    pub y: f32,
    pub mask: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bbox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
    pub confidence: f32,
    pub keypoints: Vec<KeyPoint>,
}