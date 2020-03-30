use std::{
    env, fs,
    io::{Cursor, Write},
    path,
};

use futures::{channel::mpsc, SinkExt};
use image::{
    imageops::FilterType, jpeg::JPEGEncoder, GenericImageView, ImageFormat,
};
use tonic::{Request, Response, Status};

pub use crate::pb::atwany::media_server::MediaServer;
use crate::pb::atwany::{media::*, media_server::Media};

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
            let mut image_file = fs::File::create(file_path)?;
            image_file.write_all(&res_slice_buffer.buffer)?;
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
    let dynamic_image = image::load_from_memory_with_format(&image, format)?;
    let thumbnail = dynamic_image.thumbnail(120, 120).to_bytes();
    let small = dynamic_image
        .resize(300, 250, FilterType::Triangle)
        .to_bytes();
    let medium = dynamic_image
        .resize(800, 800, FilterType::Triangle)
        .to_bytes();
    let mut buf = Cursor::new(Vec::with_capacity(image.capacity()));
    JPEGEncoder::new_with_quality(&mut buf, 80).encode(
        &image,
        dynamic_image.width(),
        dynamic_image.height(),
        dynamic_image.color(),
    )?;
    let aspect_ratio = dynamic_image.width() / dynamic_image.height(); // 16:9

    let results = vec![
        UploadResponse {
            size: Size::Thumbnail.into(),
            buffer: thumbnail,
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Small.into(),
            buffer: small,
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Medium.into(),
            buffer: medium,
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
        UploadResponse {
            size: Size::Original.into(),
            buffer: buf.into_inner(),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
        },
    ];
    Ok(results)
}

impl ToString for Size {
    fn to_string(&self) -> String {
        match self {
            Size::Small => "sm".to_string(),
            Size::Medium => "md".to_string(),
            Size::Thumbnail => "th".to_string(),
            Size::Original => "org".to_string(),
        }
    }
}
