use std::{env, fs};
use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::{Cursor, Write};
use std::path;

use futures::channel::mpsc;
use futures::SinkExt;
use image::{ColorType, GenericImageView, ImageFormat};
use image::imageops::FilterType;
use image::jpeg::JPEGEncoder;
use tonic::{Request, Response, Status};

use crate::pb::atwany::{media, media_server::Media};
use crate::pb::atwany::media::{MimeType, Size, UploadAndWriteResponse, UploadRequest, UploadResponse};
pub use crate::pb::atwany::media_server::MediaServer;

pub struct MediaService;

#[tonic::async_trait]
impl Media for MediaService {
    type UploadStream = mpsc::Receiver<Result<media::UploadResponse, Status>>;

    async fn upload(
        &self,
        request: Request<media::UploadRequest>,
    ) -> Result<Response<Self::UploadStream>, Status> {
        let req = request.into_inner();
        let (mut tx, rx) = mpsc::channel(4);
        let task = async move || -> Result<(), Status> {
            let res = process(req)
                .map_err(|e| Status::internal(e.to_string()))?;
            for res_slice in res {
                tx.send(Ok(res_slice)).await;
            }

            // TODO: Process Images Here and send them to tx

            println!("done sending");
            Ok(())
        };
        tokio::spawn(task());

        Ok(Response::new(rx))
    }

    async fn upload_and_write(&self, request: Request<UploadRequest>) -> Result<Response<UploadAndWriteResponse>, Status> {
        let req = request.into_inner();
        let mut sizes: Vec<i32> = Vec::with_capacity(4);// todo make this more dynamic
        let file_name =req.file_name.clone();
        let response_buffers = process(req);
        // let UploadResponse { aspect_ratio, file_extension } = response_buffers?[0]
        //     .clone();


        let response_buffers = response_buffers
            .map_err(|e| Status::internal(e.to_string()))?;
        let aspect_ratio = response_buffers[0].aspect_ratio.clone();
        let file_extension = response_buffers[0].file_extension.clone();
        for res_slice_buffer in response_buffers {
            let image_size = res_slice_buffer.size;
            let file_path = create_image_path(&file_name.as_str(), image_size);
            let mut image_file = fs::File::create(file_path.clone())?;
            image_file.write(&res_slice_buffer.buffer)?;
            sizes.push(res_slice_buffer.size);
        };
        let response = UploadAndWriteResponse {
            sizes,
            aspect_ratio,
            file_extension,
        };

        Ok(Response::new(response))
    }
}

fn create_image_path(file_name: &str, size: i32) -> path::PathBuf {
    let mut root = env::current_dir().unwrap();
    let path = root.join(format!("images/{}-{}.jpeg", file_name, size));
    path
}

fn process(req: media::UploadRequest) ->
anyhow::Result<Vec<media::UploadResponse>> {
    let media::UploadRequest { image, file_name, mimetype, } = req;
    let mut results: Vec<media::UploadResponse> = Vec::with_capacity(4);
    let format = match media::MimeType::from_i32(mimetype).unwrap_or_default() {
        MimeType::Png => {
            ImageFormat::Png
        }
        MimeType::Jpeg => {
            ImageFormat::Jpeg
        }
        MimeType::Gif => {
            ImageFormat::Gif
        }
        MimeType::Webp => {
            ImageFormat::WebP
        }
    };
    let dynamic_image = image::load_from_memory_with_format(&image, format)?;
    let thumbnail = dynamic_image.thumbnail(120, 120).to_bytes();
    let small = dynamic_image.resize(300, 250, FilterType::Triangle).to_bytes();
    let medium = dynamic_image.resize(800, 800, FilterType::Triangle).to_bytes();
    let mut buf = Cursor::new(Vec::with_capacity(image.capacity()));
    JPEGEncoder::new_with_quality(&mut buf, 80)
        .encode(&image, dynamic_image.width(), dynamic_image.height(), dynamic_image.color())?;
    let aspect_ratio = dynamic_image.width() / dynamic_image.height(); // 16:9

    results.push(media::UploadResponse {
        size: media::Size::Thumbnail.into(),
        buffer: thumbnail,
        file_extension: "jpeg".to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    });
    results.push(media::UploadResponse {
        size: media::Size::Small.into(),
        buffer: small,
        file_extension: "jpeg".to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    });
    results.push(media::UploadResponse {
        size: media::Size::Medium.into(),
        buffer: medium,
        file_extension: "jpeg".to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    });
    results.push(media::UploadResponse {
        size: media::Size::Original.into(),
        buffer: buf.into_inner(),
        file_extension: "jpeg".to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    });

    Ok(results)
}
// url${image}-{}.{};
