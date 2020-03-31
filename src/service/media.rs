use std::{
    env, fs,
    io::{Cursor, Write},
    path,
};

use futures::{channel::mpsc, SinkExt};
use image::{DynamicImage, GenericImageView, ImageFormat, imageops::FilterType, jpeg, jpeg::JPEGEncoder};
use prost::Message;
use tonic::{Request, Response, Status};

use crate::pb::atwany::{media::*, media_server::Media};
pub use crate::pb::atwany::media_server::MediaServer;

pub struct MediaService;

#[tonic::async_trait]
impl Media for MediaService {
    type UploadStream = mpsc::Receiver<Result<UploadResponse, Status>>;

    async fn upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<Self::UploadStream>, Status> {
        let req = request.into_inner();
        let (mut tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let res = process(req).unwrap();
            for res_slice in res {
                tx.send(Ok(res_slice)).await.unwrap();
            }
        });
        Ok(Response::new(rx))
    }

    async fn upload_and_write(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadAndWriteResponse>, Status> {
        let req = request.into_inner();
        let mut sizes: Vec<i32> = Vec::with_capacity(4); // todo make this more dynamic
        let file_name = req.file_name.clone();
        let response_buffers =
            process(req).map_err(|e| Status::internal(e.to_string()))?;
        let aspect_ratio = response_buffers[0].aspect_ratio.clone();
        let file_extension = response_buffers[0].file_extension.clone();
        for res_slice_buffer in response_buffers {
            let image_size = Size::from_i32(res_slice_buffer.size).unwrap();
            let file_path = create_image_path(&file_name.as_str(), image_size);
            let mut image_file = fs::File::create(file_path)
                .map_err(|e| Status::internal(e.to_string()))?;
            image_file
                .write_all(&res_slice_buffer.buffer)
                .map_err(|e| Status::internal(e.to_string()))?;
            image_file
                .flush()
                .map_err(|e| Status::internal(e.to_string()))?;
            sizes.push(res_slice_buffer.size);
        }
        let response = UploadAndWriteResponse {
            sizes,
            aspect_ratio,
            file_extension,
        };
        Ok(Response::new(response))
    }
}

fn create_image_path(file_name: &str, size: Size) -> path::PathBuf {
    let root = env::current_dir().unwrap();
    root.join(format!("images/{}_{}.jpeg", file_name, size.to_string()))
}

fn process(req: UploadRequest) -> anyhow::Result<Vec<UploadResponse>> {
    let UploadRequest {
        image, mimetype, ..
    } = req;
    let format = match MimeType::from_i32(mimetype).unwrap_or_default() {
        MimeType::Png => ImageFormat::Png,
        MimeType::Jpeg => ImageFormat::Jpeg,
        MimeType::Gif => ImageFormat::Gif,
        MimeType::Webp => ImageFormat::WebP,
    };
    let dynamic_image = match image::load_from_memory_with_format(&image, format) {
        Ok(image_data) => image_data,
        Err(e) => {
            dbg!(e.to_string());
            image::load_from_memory(&image).unwrap()
        }
    };
    let placeholder = dynamic_image.thumbnail(20, 20);
    let thumbnail = dynamic_image.thumbnail(200, 200);
    let small = dynamic_image
        .resize(
            400,
            400,
            FilterType::Triangle,
        );
    let medium = dynamic_image
        .resize(
            dynamic_image.width(),
            dynamic_image.height(),
            FilterType::Triangle,
        );
    let aspect_ratio = dynamic_image.width() / dynamic_image.height(); // 16:9

    let results = vec![
        UploadResponse {
            size: Size::Thumbnail.into(),
            buffer: get_image_bytes(thumbnail),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Thumbnail.into(),
            buffer: get_image_bytes(placeholder),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Small.into(),
            buffer: get_image_bytes(small),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Medium.into(),
            buffer: get_image_bytes(medium),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Original.into(),
            buffer: image,
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
    ];
    Ok(results)
}

fn get_image_bytes(image: DynamicImage) -> Vec<u8> {
    let mut output = Vec::new();
    let mut j = jpeg::JPEGEncoder::new_with_quality(&mut output, 20);
    j.encode(&image.to_bytes(), image.width(), image.height(), image.color()).unwrap();
    output
}

impl ToString for Size {
    fn to_string(&self) -> String {
        match self {
            Size::Small => "sm-400".to_string(),
            Size::Thumbnail => "th-200".to_string(),
            Size::Placeholder => "th-20".to_string(),
            Size::Medium => "md".to_string(),
            Size::Original => "org".to_string(),
        }
    }
}
