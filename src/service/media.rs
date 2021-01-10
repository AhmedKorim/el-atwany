use std::{
	env, fs,
	io::{Cursor, Write},
	path,
};
use std::sync::Arc;

use futures::{channel::mpsc, SinkExt, StreamExt};
use image::{
	DynamicImage, GenericImageView, ImageFormat, imageops::FilterType,
	jpeg, jpeg::JPEGEncoder,
};
use prost::Message;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::pb::atwany::{
	media::{*, upload_and_write_response::MediaSize},
	media_server::Media,
};
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
			let res = process(req).await.unwrap();
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
		dbg!("uploading {}" , &file_name);
		dbg!("getting response buffers");
		let response_buffers =
			process(req).await.map_err(|e| Status::internal(e.to_string()))?;
		dbg!("Got response_buffers");
		let mut media_meta: Vec<MediaSize> =
			Vec::with_capacity(response_buffers.capacity()); // todo make this
		// more dynamic
		let aspect_ratio = response_buffers[0].aspect_ratio.clone();
		let file_extension = response_buffers[0].file_extension.clone();
		// let img_for_blur_hashing = image::load_from_memory(&response_buffers[0].buffer).unwrap();
		// let (width, height) = img_for_blur_hashing.dimensions();
		// let blur_hash = blurhash::encode(4, 3, width, height, &img_for_blur_hashing.to_rgba().into_vec());
		let blur_hash="3000asdfk".to_string();
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

pub fn blur() {
	let path = create_file_path(
		"119890471_1787496678077342_6268729416129342664_o",
		"jpg",
	);
	let img = image::open(path).unwrap();
	let img = img.thumbnail(30, 30);
	let (width, height) = img.dimensions();
	let blurhash = blurhash::encode(4, 3, width, height, &img.to_rgba().into_vec());
	dbg!(blurhash);
}

async fn process(req: UploadRequest) -> Result<Vec<UploadResponse>, String> {
	let UploadRequest {
		image, mimetype, ..
	} = req;
	let format = match MimeType::from_i32(mimetype).unwrap_or_default() {
		MimeType::Png => ImageFormat::Png,
		MimeType::Jpeg => ImageFormat::Jpeg,
		MimeType::Gif => ImageFormat::Gif,
		MimeType::Webp => ImageFormat::WebP,
	};
	let sizes: Vec<Size> = vec![Size::Medium, Size::Placeholder, Size::Small, Size::Thumbnail];
	let dynamic_image =
		match image::load_from_memory_with_format(&image, format) {
			Ok(image_data) => image_data,
			Err(e) => {
				dbg!(e.to_string());
				image::load_from_memory(&image).unwrap()
			}
		};
	dbg!("building response");
	let res: Arc<Mutex<Vec<UploadResponse>>> = Arc::new(Mutex::new(vec![]));
	let aspect_ratio = dynamic_image.width() / dynamic_image.height(); // 16:9
	let mut handles = vec![];
	for (index, size) in sizes.iter().enumerate() {
		dbg!("loop {}" ,index);
		let r = res.clone();
		let i = dynamic_image.clone();
		let size = size.clone();
		handles.push(tokio::spawn(async move {
			dbg!("generating {}" , size);

			let image_size: u32 = match size {
				Size::Original => { unreachable!() }
				Size::Placeholder => { 20 }
				Size::Thumbnail => { 64 }
				Size::Small => { 600 }
				Size::Medium => { 400 }
			};
			let img = i.thumbnail(image_size, image_size);
			let res = UploadResponse {
				size: size.into(),
				buffer: get_image_bytes(&img),
				file_extension: "jpeg".to_string(),
				aspect_ratio: aspect_ratio.to_string(),
				width: img.width().into(),
				height: img.height().into(),
				url_suffix: size.to_string(),
			};
			let mut list = r.lock().await;
			list.push(res);
			dbg!("generated {}" , size);
		}));
	}
	futures::future::join_all(handles).await;
	let res = res.lock().await;
	let mut results = vec![
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
	for x in res.iter() {
		results.push(x.clone())
	}

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
