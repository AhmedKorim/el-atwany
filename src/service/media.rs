use std::{
    env, fs,
    io::{Cursor, Write},
    path,
};

use futures::{channel::mpsc, SinkExt, StreamExt};
use image::{
    DynamicImage, GenericImageView, ImageFormat, imageops::FilterType,
    jpeg, jpeg::JPEGEncoder,
};
use prost::Message;
use tonic::{Request, Response, Status};

use crate::pb::atwany::{
    media::{*, upload_and_write_response::MediaSize},
    media_server::Media,
};
pub use crate::pb::atwany::media_server::MediaServer;
use tokio::task::JoinHandle;
use futures::core_reexport::ops::Deref;

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
        let img = image::load_from_memory(&req.image)
            .map_err(|_| {
                Status::internal("Failed to obtain image for blur hashing")
            })?;

        tokio::spawn(async move {
            let res = process(img).await.unwrap();
            for res_slice in res {
                tx.send(Ok(res_slice)).await.unwrap();
            }
        });
        Ok(Response::new(rx))
    }

    async fn upload_file(
        &self,
        request: Request<FileUpload>,
    ) -> Result<Response<FileUploadResponse>, Status> {
        let req = request.into_inner();
        let file_name = req.file_name.clone();
        let ext = req.file_extension.clone();
        let path = create_file_path(&file_name, &ext);
        let mut file = fs::File::create(path)
            .map_err(|e| Status::internal(e.to_string()))?;
        file.write_all(&req.file)
            .map_err(|e| Status::internal(e.to_string()))?;
        file.flush().map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(FileUploadResponse {
            file_extension: ext,
        }))
    }

    async fn upload_and_write(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadAndWriteResponse>, Status> {
        let req = request.into_inner();
        let file_name = req.file_name.clone();
        let img = image::load_from_memory(&req.image)
            .map_err(|_| {
                Status::internal("Failed to obtain image for blur hashing")
            })?;

        let (width, height) = img.dimensions();
        let blur_hash = blurhash::encode(4, 3, width, height, &img.to_rgba().into_vec());


        let response_buffers =
            process(img).await.map_err(|_| Status::internal("Something went wrong"))?;
        let mut media_meta: Vec<MediaSize> =
            Vec::with_capacity(response_buffers.capacity());
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
            blur_hash,
        };
        Ok(Response::new(response))
    }
}

fn create_image_path(file_name: &str, size: Size) -> path::PathBuf {
    let root = env::current_dir().unwrap();
    root.join(format!("images/{}_{}.jpeg", file_name, size.to_string()))
}

fn create_file_path(file_name: &str, ext: &str) -> path::PathBuf {
    let root = env::current_dir().unwrap();
    root.join(format!("files/{}.{}", file_name, ext.to_string()))
}


const SIZE: [Size; 4] = [Size::Medium, Size::Placeholder, Size::Small, Size::Thumbnail];

async fn process(image: DynamicImage) -> anyhow::Result<Vec<UploadResponse>, ()> {
    let mut images: Vec<JoinHandle<UploadResponse>> = Vec::with_capacity(5);
    let aspect_ratio = image.width() / image.height(); // 16:9

    for size in SIZE.iter() {
        let image = image.clone();
        let aspect_ratio = aspect_ratio.clone();
        images.push(tokio::spawn(async move {
            let dim = match size {
                Size::Placeholder => { 64 }
                Size::Thumbnail => { 200 }
                Size::Small => { 400 }
                Size::Medium => { 800 }
                _ => {
                    unreachable!()
                }
            };
            let image = image.thumbnail(dim, dim);
            UploadResponse {
                size: size.clone().into(),
                buffer: get_image_bytes(&image),
                file_extension: "jpeg".to_string(),
                aspect_ratio: aspect_ratio.to_string(),
                width: image.width().into(),
                height: image.height().into(),
                url_suffix: size.to_string(),
            }
        }))
    }
    let mut images: Vec<_> = futures::future::join_all(images).await.into_iter().flatten().collect();
    let mut results = vec![
        UploadResponse {
            size: Size::Original.into(),
            buffer: get_image_bytes(&image),
            file_extension: "jpeg".to_string(),
            aspect_ratio: aspect_ratio.to_string(),
            width: image.width().into(),
            height: image.height().into(),
            url_suffix: Size::Original.to_string(),
        },
    ];
    results.append(&mut images);
    Ok(results)
}

fn get_image_bytes(image: &DynamicImage) -> Vec<u8> {
    let mut output = Vec::new();
    let mut j = jpeg::JPEGEncoder::new_with_quality(&mut output, 20);
    j.encode(
        &image.to_bytes(),
        image.width(),
        image.height(),
        image.color(),
    )
        .unwrap();
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
