use candle_core::{ DType, Device, IndexOp, Result, Tensor };
use candle_nn::{ Module, VarBuilder, conv2d_no_bias, batch_norm, Conv2dConfig, Conv2d, conv2d };
use serde_json::Value;
use std::io::Cursor;
use crate::model::model_structs::structs::{
    Bottleneck, C2f, ConvBlock, DarkNet, DetectionHead, DetectionHeadOut, Dfl, Multiples, Sppf, Upsample, YoloV8, YoloV8Neck,
    Model, ModelData
};
use crate::model::model_functions::functions::{ dist2bbox, make_anchors, report_detect };
use crate::model::classes::YOLO_CLASSES;

impl Model {

    /* <--- ADD FUNCTIONS BASED ON YOUR NEEDS START HERE ---> */

    pub fn run_img_detect(
        &self,
        image_data: Vec<u8>,
        conf_threshold: f32,
        iou_threshold: f32,
    ) -> Result<Value> {

        // Convert the image data into an in-memory image format
        let image_data = Cursor::new(image_data);
        let original_image = image::ImageReader::new(image_data)
            .with_guessed_format()?
            .decode()
            .map_err(candle_core::Error::wrap)?;

        // Resize the image while maintaining the aspect ratio
        let (width, height) = {
            let w = original_image.width() as usize;
            let h = original_image.height() as usize;
            if w < h {
                let w = w * 640 / h;
                (w / 32 * 32, 640) // Ensure the width is divisible by 32
            } else {
                let h = h * 640 / w;
                (640, h / 32 * 32) // Ensure the height is divisible by 32
            }
        };

        // Resize the image and convert it to a tensor
        let image_t = {
            let img = original_image.resize_exact(
                width as u32,
                height as u32,
                image::imageops::FilterType::CatmullRom,
            );
            let data = img.to_rgb8().into_raw();
            Tensor::from_vec(
                data,
                (img.height() as usize, img.width() as usize, 3),
                &Device::Cpu,
            )?
            .permute((2, 0, 1))? // Convert to tensor with channel-first format (C, H, W)
        };

        let image_t = (image_t.unsqueeze(0)?.to_dtype(DType::F32)? * (1. / 255.))?; // Normalize to 0-1 range

        // Run the model on the input image
        let predictions = self.model.forward(&image_t)?.squeeze(0)?;

        // Extract bounding boxes from the model's predictions
        let bboxes: Value = report_detect(
            &predictions,
            original_image,
            width,
            height,
            conf_threshold,
            iou_threshold,
        ).expect("Failed bboxes report detect");
        // Use the report_detect function to process the predictions and get detected classes
        Ok(bboxes)
    }
    
    // pub fn run_img_classification(
    //     &self,
    //     image_data: Vec<u8>,
    //     conf_threshold: f32
    // ) -> Result<String> {
    //     // Convert the image data into an in-memory image format
    //     let image_data = Cursor::new(image_data);
    //     let original_image = image::ImageReader::new(image_data)
    //         .with_guessed_format()?
    //         .decode()
    //         .map_err(candle_core::Error::wrap)?;

    //     // Resize the image while maintaining the aspect ratio
    //     let (width, height) = {
    //         let w = original_image.width() as usize;
    //         let h = original_image.height() as usize;
    //         if w < h {
    //             let w = w * 640 / h;
    //             (w / 32 * 32, 640) // Ensure the width is divisible by 32
    //         } else {
    //             let h = h * 640 / w;
    //             (640, h / 32 * 32) // Ensure the height is divisible by 32
    //         }
    //     };

    //     // Resize the image and convert it to a tensor
    //     let image_t = {
    //         let img = original_image.resize_exact(
    //             width as u32,
    //             height as u32,
    //             image::imageops::FilterType::CatmullRom,
    //         );
    //         let data = img.to_rgb8().into_raw();
    //         Tensor::from_vec(
    //             data,
    //             (img.height() as usize, img.width() as usize, 3),
    //             &Device::Cpu,
    //         )?
    //         .permute((2, 0, 1))? // Convert to tensor with channel-first format (C, H, W)
    //     };

    //     let image_t = (image_t.unsqueeze(0)?.to_dtype(DType::F32)? * (1. / 255.))?; // Normalize to 0-1 range

    //     // Run the model on the input image
    //     let predictions = self.model.forward(&image_t)?.squeeze(0)?;

    //     // Use the report_detect function to process the predictions and get detected classes
    //     let detected_classes = report_classified(&predictions, conf_threshold)?;

    //     Ok(detected_classes)
    // }
    
    /* <--- ADD FUNCTIONS BASED ON YOUR NEEDS ENDS HERE ---> */

    pub fn load(md: ModelData) -> Result<Self> {
        let multiples = match md.model_size.as_str() {
            "n" => Multiples::n(),
            "s" => Multiples::s(),
            "m" => Multiples::m(),
            "l" => Multiples::l(),
            "x" => Multiples::x(),
            _ => Err(candle_core::Error::Msg(
                "invalid model size: must be n, s, m, l or x".to_string(),
            ))?,
        };

        let dev = &Device::Cpu;
        let vb = VarBuilder::from_buffered_safetensors(md.weights, DType::F32, dev)?;
        let model = YoloV8::load(vb, multiples, YOLO_CLASSES.len())?;
        Ok(Self { model })
    }
}
/* 
    THIS MULTIPLES IS BASED ON WHAT VERSION MODELS USED AND THE REST CODE BELOW IS FOR COMPUTATION.
    YoloV8 (n) - STANDS FOR NANO
    you can read the documentations here:
        https://github.com/ultralytics/ultralytics
*/
impl Multiples {
    pub fn n() -> Self {
        Self {
            depth: 0.33,
            width: 0.25,
            ratio: 2.0,
        }
    }
    pub fn s() -> Self {
        Self {
            depth: 0.33,
            width: 0.50,
            ratio: 2.0,
        }
    }
    pub fn m() -> Self {
        Self {
            depth: 0.67,
            width: 0.75,
            ratio: 1.5,
        }
    }
    pub fn l() -> Self {
        Self {
            depth: 1.00,
            width: 1.00,
            ratio: 1.0,
        }
    }
    pub fn x() -> Self {
        Self {
            depth: 1.00,
            width: 1.25,
            ratio: 1.0,
        }
    }

    fn filters(&self) -> (usize, usize, usize) {
        let f1 = (256. * self.width) as usize;
        let f2 = (512. * self.width) as usize;
        let f3 = (512. * self.width * self.ratio) as usize;
        (f1, f2, f3)
    }
}

impl Upsample {
    fn new(scale_factor: usize) -> Result<Self> {
        Ok(Upsample { scale_factor })
    }
}

impl Module for Upsample {
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let (_b_size, _channels, h, w) = xs.dims4()?;
        xs.upsample_nearest2d(self.scale_factor * h, self.scale_factor * w)
    }
}

impl ConvBlock {
    fn load(
        vb: VarBuilder,
        c1: usize,
        c2: usize,
        k: usize,
        stride: usize,
        padding: Option<usize>,
    ) -> Result<Self> {
        let padding = padding.unwrap_or(k / 2);
        let cfg = Conv2dConfig {
            padding,
            stride,
            groups: 1,
            dilation: 1,
        };
        let conv = conv2d_no_bias(c1, c2, k, cfg, vb.pp("conv"))?;
        let bn = batch_norm(c2, 1e-3, vb.pp("bn"))?;
        Ok(Self { conv, bn })
    }
}

impl Module for ConvBlock {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let xs = self.conv.forward(xs)?.apply_t(&self.bn, false)?;
        candle_nn::ops::silu(&xs)
    }
}

impl Bottleneck {
    fn load(vb: VarBuilder, c1: usize, c2: usize, shortcut: bool) -> Result<Self> {
        let channel_factor = 1.;
        let c_ = (c2 as f64 * channel_factor) as usize;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, c_, 3, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), c_, c2, 3, 1, None)?;
        let residual = c1 == c2 && shortcut;
        Ok(Self { cv1, cv2, residual })
    }
}

impl Module for Bottleneck {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let ys = self.cv2.forward(&self.cv1.forward(xs)?)?;
        if self.residual {
            xs + ys
        } else {
            Ok(ys)
        }
    }
}

impl C2f {
    fn load(vb: VarBuilder, c1: usize, c2: usize, n: usize, shortcut: bool) -> Result<Self> {
        let c = (c2 as f64 * 0.5) as usize;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, 2 * c, 1, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), (2 + n) * c, c2, 1, 1, None)?;
        let mut bottleneck = Vec::with_capacity(n);
        for idx in 0..n {
            let b = Bottleneck::load(vb.pp(format!("bottleneck.{idx}")), c, c, shortcut)?;
            bottleneck.push(b)
        }
        Ok(Self {
            cv1,
            cv2,
            bottleneck,
        })
    }
}

impl Module for C2f {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let ys = self.cv1.forward(xs)?;
        let mut ys = ys.chunk(2, 1)?;
        for m in self.bottleneck.iter() {
            ys.push(m.forward(ys.last().unwrap())?)
        }
        let zs = Tensor::cat(ys.as_slice(), 1)?;
        self.cv2.forward(&zs)
    }
}

impl Sppf {
    fn load(vb: VarBuilder, c1: usize, c2: usize, k: usize) -> Result<Self> {
        let c_ = c1 / 2;
        let cv1 = ConvBlock::load(vb.pp("cv1"), c1, c_, 1, 1, None)?;
        let cv2 = ConvBlock::load(vb.pp("cv2"), c_ * 4, c2, 1, 1, None)?;
        Ok(Self { cv1, cv2, k })
    }
}

impl Module for Sppf {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (_, _, _, _) = xs.dims4()?;
        let xs = self.cv1.forward(xs)?;
        let xs2 = xs
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        let xs3 = xs2
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        let xs4 = xs3
            .pad_with_zeros(2, self.k / 2, self.k / 2)?
            .pad_with_zeros(3, self.k / 2, self.k / 2)?
            .max_pool2d_with_stride(self.k, 1)?;
        self.cv2.forward(&Tensor::cat(&[&xs, &xs2, &xs3, &xs4], 1)?)
    }
}

impl Dfl {
    fn load(vb: VarBuilder, num_classes: usize) -> Result<Self> {
        let conv = conv2d_no_bias(num_classes, 1, 1, Default::default(), vb.pp("conv"))?;
        Ok(Self { conv, num_classes })
    }
}


impl Module for Dfl {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (b_sz, _channels, anchors) = xs.dims3()?;
        let xs = xs
            .reshape((b_sz, 4, self.num_classes, anchors))?
            .transpose(2, 1)?;
        let xs = candle_nn::ops::softmax(&xs, 1)?;
        self.conv.forward(&xs)?.reshape((b_sz, 4, anchors))
    }
}

impl DarkNet {
    fn load(vb: VarBuilder, m: Multiples) -> Result<Self> {
        let (w, r, d) = (m.width, m.ratio, m.depth);
        let b1_0 = ConvBlock::load(vb.pp("b1.0"), 3, (64. * w) as usize, 3, 2, Some(1))?;
        let b1_1 = ConvBlock::load(
            vb.pp("b1.1"),
            (64. * w) as usize,
            (128. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b2_0 = C2f::load(
            vb.pp("b2.0"),
            (128. * w) as usize,
            (128. * w) as usize,
            (3. * d).round() as usize,
            true,
        )?;
        let b2_1 = ConvBlock::load(
            vb.pp("b2.1"),
            (128. * w) as usize,
            (256. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b2_2 = C2f::load(
            vb.pp("b2.2"),
            (256. * w) as usize,
            (256. * w) as usize,
            (6. * d).round() as usize,
            true,
        )?;
        let b3_0 = ConvBlock::load(
            vb.pp("b3.0"),
            (256. * w) as usize,
            (512. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let b3_1 = C2f::load(
            vb.pp("b3.1"),
            (512. * w) as usize,
            (512. * w) as usize,
            (6. * d).round() as usize,
            true,
        )?;
        let b4_0 = ConvBlock::load(
            vb.pp("b4.0"),
            (512. * w) as usize,
            (512. * w * r) as usize,
            3,
            2,
            Some(1),
        )?;
        let b4_1 = C2f::load(
            vb.pp("b4.1"),
            (512. * w * r) as usize,
            (512. * w * r) as usize,
            (3. * d).round() as usize,
            true,
        )?;
        let b5 = Sppf::load(
            vb.pp("b5.0"),
            (512. * w * r) as usize,
            (512. * w * r) as usize,
            5,
        )?;
        Ok(Self {
            b1_0,
            b1_1,
            b2_0,
            b2_1,
            b2_2,
            b3_0,
            b3_1,
            b4_0,
            b4_1,
            b5,
        })
    }

    fn forward(&self, xs: &Tensor) -> Result<(Tensor, Tensor, Tensor)> {
        let x1 = self.b1_1.forward(&self.b1_0.forward(xs)?)?;
        let x2 = self
            .b2_2
            .forward(&self.b2_1.forward(&self.b2_0.forward(&x1)?)?)?;
        let x3 = self.b3_1.forward(&self.b3_0.forward(&x2)?)?;
        let x4 = self.b4_1.forward(&self.b4_0.forward(&x3)?)?;
        let x5 = self.b5.forward(&x4)?;
        Ok((x2, x3, x5))
    }
}

impl YoloV8Neck {
    fn load(vb: VarBuilder, m: Multiples) -> Result<Self> {
        let up = Upsample::new(2)?;
        let (w, r, d) = (m.width, m.ratio, m.depth);
        let n = (3. * d).round() as usize;
        let n1 = C2f::load(
            vb.pp("n1"),
            (512. * w * (1. + r)) as usize,
            (512. * w) as usize,
            n,
            false,
        )?;
        let n2 = C2f::load(
            vb.pp("n2"),
            (768. * w) as usize,
            (256. * w) as usize,
            n,
            false,
        )?;
        let n3 = ConvBlock::load(
            vb.pp("n3"),
            (256. * w) as usize,
            (256. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let n4 = C2f::load(
            vb.pp("n4"),
            (768. * w) as usize,
            (512. * w) as usize,
            n,
            false,
        )?;
        let n5 = ConvBlock::load(
            vb.pp("n5"),
            (512. * w) as usize,
            (512. * w) as usize,
            3,
            2,
            Some(1),
        )?;
        let n6 = C2f::load(
            vb.pp("n6"),
            (512. * w * (1. + r)) as usize,
            (512. * w * r) as usize,
            n,
            false,
        )?;
        Ok(Self {
            up,
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
        })
    }

    fn forward(&self, p3: &Tensor, p4: &Tensor, p5: &Tensor) -> Result<(Tensor, Tensor, Tensor)> {
        let x = self
            .n1
            .forward(&Tensor::cat(&[&self.up.forward(p5)?, p4], 1)?)?;
        let head_1 = self
            .n2
            .forward(&Tensor::cat(&[&self.up.forward(&x)?, p3], 1)?)?;
        let head_2 = self
            .n4
            .forward(&Tensor::cat(&[&self.n3.forward(&head_1)?, &x], 1)?)?;
        let head_3 = self
            .n6
            .forward(&Tensor::cat(&[&self.n5.forward(&head_2)?, p5], 1)?)?;
        Ok((head_1, head_2, head_3))
    }
}

impl DetectionHead {
    fn load(vb: VarBuilder, nc: usize, filters: (usize, usize, usize)) -> Result<Self> {
        let ch = 16;
        let dfl = Dfl::load(vb.pp("dfl"), ch)?;
        let c1 = usize::max(filters.0, nc);
        let c2 = usize::max(filters.0 / 4, ch * 4);
        let cv3 = [
            Self::load_cv3(vb.pp("cv3.0"), c1, nc, filters.0)?,
            Self::load_cv3(vb.pp("cv3.1"), c1, nc, filters.1)?,
            Self::load_cv3(vb.pp("cv3.2"), c1, nc, filters.2)?,
        ];
        let cv2 = [
            Self::load_cv2(vb.pp("cv2.0"), c2, ch, filters.0)?,
            Self::load_cv2(vb.pp("cv2.1"), c2, ch, filters.1)?,
            Self::load_cv2(vb.pp("cv2.2"), c2, ch, filters.2)?,
        ];
        let no = nc + ch * 4;
        Ok(Self {
            dfl,
            cv2,
            cv3,
            ch,
            no,
        })
    }

    fn load_cv3(
        vb: VarBuilder,
        c1: usize,
        nc: usize,
        filter: usize,
    ) -> Result<(ConvBlock, ConvBlock, Conv2d)> {
        let block0 = ConvBlock::load(vb.pp("0"), filter, c1, 3, 1, None)?;
        let block1 = ConvBlock::load(vb.pp("1"), c1, c1, 3, 1, None)?;
        let conv = conv2d(c1, nc, 1, Default::default(), vb.pp("2"))?;
        Ok((block0, block1, conv))
    }

    fn load_cv2(
        vb: VarBuilder,
        c2: usize,
        ch: usize,
        filter: usize,
    ) -> Result<(ConvBlock, ConvBlock, Conv2d)> {
        let block0 = ConvBlock::load(vb.pp("0"), filter, c2, 3, 1, None)?;
        let block1 = ConvBlock::load(vb.pp("1"), c2, c2, 3, 1, None)?;
        let conv = conv2d(c2, 4 * ch, 1, Default::default(), vb.pp("2"))?;
        Ok((block0, block1, conv))
    }

    fn forward(&self, xs0: &Tensor, xs1: &Tensor, xs2: &Tensor) -> Result<DetectionHeadOut> {
        let forward_cv = |xs, i: usize| {
            let xs_2 = self.cv2[i].0.forward(xs)?;
            let xs_2 = self.cv2[i].1.forward(&xs_2)?;
            let xs_2 = self.cv2[i].2.forward(&xs_2)?;

            let xs_3 = self.cv3[i].0.forward(xs)?;
            let xs_3 = self.cv3[i].1.forward(&xs_3)?;
            let xs_3 = self.cv3[i].2.forward(&xs_3)?;
            Tensor::cat(&[&xs_2, &xs_3], 1)
        };
        let xs0 = forward_cv(xs0, 0)?;
        let xs1 = forward_cv(xs1, 1)?;
        let xs2 = forward_cv(xs2, 2)?;

        let (anchors, strides) = make_anchors(&xs0, &xs1, &xs2, (8, 16, 32), 0.5)?;
        let anchors = anchors.transpose(0, 1)?.unsqueeze(0)?;
        let strides = strides.transpose(0, 1)?;

        let reshape = |xs: &Tensor| {
            let d = xs.dim(0)?;
            let el = xs.elem_count();
            xs.reshape((d, self.no, el / (d * self.no)))
        };
        let ys0 = reshape(&xs0)?;
        let ys1 = reshape(&xs1)?;
        let ys2 = reshape(&xs2)?;

        let x_cat = Tensor::cat(&[ys0, ys1, ys2], 2)?;
        let box_ = x_cat.i((.., ..self.ch * 4))?;
        let cls = x_cat.i((.., self.ch * 4..))?;

        let dbox = dist2bbox(&self.dfl.forward(&box_)?, &anchors)?;
        let dbox = dbox.broadcast_mul(&strides)?;
        let pred = Tensor::cat(&[dbox, candle_nn::ops::sigmoid(&cls)?], 1)?;
        Ok(DetectionHeadOut {
            pred,
            anchors,
            strides,
        })
    }
}

impl YoloV8 {
    fn load(vb: VarBuilder, m: Multiples, num_classes: usize) -> Result<Self> {
        let net = DarkNet::load(vb.pp("net"), m)?;
        let fpn = YoloV8Neck::load(vb.pp("fpn"), m)?;
        let head = DetectionHead::load(vb.pp("head"), num_classes, m.filters())?;
        Ok(Self { net, fpn, head })
    }
}

impl Module for YoloV8 {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (xs1, xs2, xs3) = self.net.forward(xs)?;
        let (xs1, xs2, xs3) = self.fpn.forward(&xs1, &xs2, &xs3)?;
        Ok(self.head.forward(&xs1, &xs2, &xs3)?.pred)
    }
}
