#![deny(
unsafe_code,
missing_debug_implementations,
missing_copy_implementations,
elided_lifetimes_in_paths,
rust_2018_idioms,
clippy::fallible_impl_from,
clippy::missing_const_for_fn
)]
#![feature(async_closure)]

use async_ctrlc::CtrlC;
use log::info;
use std::env;
use tonic::transport::Server;

mod sora;
mod pb;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "atwany");
//    dotenv::dotenv()?;
    pretty_env_logger::init_timed();
    let addr = "0.0.0.0:50051".parse()?;
    info!("Starting Server on {}", addr);
    let svc = service::MediaServer::new(service::MediaService);
    Server::builder()
        .concurrency_limit_per_connection(100)
        .tcp_nodelay(true)
        .add_service(svc)
        .serve_with_shutdown(addr, CtrlC::new()?)
        .await?;
    info!("Shutdown ..");
    Ok(())
}
