use std::convert::From;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use image::{GenericImageView, ImageFormat, jpeg};
use image::imageops::FilterType;

use crate::pb::atwany::media::{MimeType, Size};

// trait MediaImage {
//     fn new(image: Vec<u8>, format: MimeType, filename: &str) -> Self;
// }
//
// #[derive(Debug)]
// struct Sora {
//     sizes: Size,
//     mimetype: ImageFormat,
//     filename: String,
// }
//
// impl MediaImage for Sora {
//     fn new(image: Vec<u8>, format: MimeType, filename: &str) -> Self {
//         Sora {
//             mimetype:
//         }
//     }
// }

pub fn run_track() {
    let img = image::open("images/onsyt1580768949301.png").unwrap();

    let scaled = img.resize(img.width(), img.height(), FilterType::Gaussian);

    let mut output = File::create(&format!("images/co/test-original.png")).unwrap();
    let mut j = jpeg::JPEGEncoder::new_with_quality(&mut output, 80);
    j.encode(&scaled.to_bytes(), scaled.width(), scaled.height(), scaled.color()).unwrap();


    for size in &[20_u32, 40, 100, 200, 300, 400] {
        let scaled = img.thumbnail(*size, *size);
        let mut output = File::create(format!("images/co/test-thumb{}.png", size)).unwrap();
        let mut j = jpeg::JPEGEncoder::new_with_quality(&mut output, 80);
        j.encode(&scaled.to_bytes(), scaled.width(), scaled.height(), scaled.color()).unwrap();
    }
} // test{  }.jpeg
