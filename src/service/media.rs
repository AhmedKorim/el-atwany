pub use crate::pb::atwany::media_server::MediaServer;
use crate::pb::atwany::{media, media_server::Media};
use futures::channel::mpsc;
use tonic::{Request, Response, Status};
pub struct MediaService;

#[tonic::async_trait]
impl Media for MediaService {
    type UploadStream = mpsc::Receiver<Result<media::UploadResponse, Status>>;

    async fn upload(
        &self,
        _request: Request<media::UploadRequest>,
    ) -> Result<Response<Self::UploadStream>, Status> {
        let (_tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            // TODO: Process Images Here and send them to tx
            println!("done sending");
        });

        Ok(Response::new(rx))
    }
}
