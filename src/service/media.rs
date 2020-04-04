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
use crate::pb::atwany::media::upload_and_write_response::MediaSize;

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
        let file_name = req.file_name.clone();
        let response_buffers =
            process(req).map_err(|e| Status::internal(e.to_string()))?;
        let mut media_meta: Vec<MediaSize> = Vec::with_capacity(response_buffers.capacity()); // todo make this
        // more dynamic
        let aspect_ratio = response_buffers[0].aspect_ratio.clone();
        let file_extension = response_buffers[0].file_extension.clone();
        for res_slice_buffer in response_buffers {
            let image_size = Size::from_i32(res_slice_buffer.size).unwrap();
            let file_path = create_image_path(&file_name.as_str(), image_size);
            dbg!(&file_name);
            dbg!(&file_path);
            let mut image_file = fs::File::create(file_path)
                .map_err(|e| Status::internal(e.to_string()))?;
            image_file
                .write_all(&res_slice_buffer.buffer)
                .map_err(|e| Status::internal(e.to_string()))?;
            image_file
                .flush()
                .map_err(|e| Status::internal(e.to_string()))?;
            dbg!(res_slice_buffer.size);
            media_meta.push(MediaSize {
                height: res_slice_buffer.height,
                width: res_slice_buffer.width,
                size: res_slice_buffer.size,
                url_suffix: res_slice_buffer.url_suffix,
            })
        }
        let response = UploadAndWriteResponse {
            aspect_ratio,
            file_extension,
            media_meta,
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
    dbg!(aspect_ratio);
    dbg!(format!(" dynamic_image width {} height {}", dynamic_image.width(), dynamic_image.height
    ()));
    dbg!(format!("small width {} height {}", small.width(), small.height()));
    dbg!(format!("medium width {} height {}", medium.width(), medium.height()));
    let results = vec![
        UploadResponse {
            size: Size::Thumbnail.into(),
            buffer: get_image_bytes(&thumbnail),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: thumbnail.width().into(),
            height: thumbnail.height().into(),
            url_suffix: Size::Thumbnail.to_string(),
        },
        UploadResponse {
            size: Size::Placeholder.into(),
            buffer: get_image_bytes(&placeholder),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: placeholder.width().into(),
            height: placeholder.height().into(),
            url_suffix: Size::Placeholder.to_string(),
        },
        UploadResponse {
            size: Size::Small.into(),
            buffer: get_image_bytes(&small),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: small.width().into(),
            height: small.height().into(),
            url_suffix: Size::Small.to_string(),
        },
        UploadResponse {
            size: Size::Medium.into(),
            buffer: get_image_bytes(&medium),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: medium.width().into(),
            height: medium.height().into(),
            url_suffix: Size::Medium.to_string(),

        },
        UploadResponse {
            size: Size::Original.into(),
            buffer: image,
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: dynamic_image.width().into(),
            height: dynamic_image.height().into(),
            url_suffix: Size::Original.to_string(),
        },
    ];
    Ok(results)
}

fn get_image_bytes(image: &DynamicImage) -> Vec<u8> {
    let mut output = Vec::new();
    let mut j = jpeg::JPEGEncoder::new_with_quality(&mut output, 20);
    j.encode(&image.to_bytes(), image.width(), image.height(), image.color()).unwrap();
    output
}

// fn get_aspect_ratio(width: i32, height: i32) -> String {
//     // 600/450 =>
// }

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
