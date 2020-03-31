#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Media {}
pub mod media {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UploadAndWriteResponse {
        #[prost(enumeration = "Size", repeated, tag = "1")]
        pub sizes: ::std::vec::Vec<i32>,
        #[prost(string, tag = "3")]
        pub file_extension: std::string::String,
        #[prost(string, tag = "4")]
        pub aspect_ratio: std::string::String,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UploadResponse {
        #[prost(enumeration = "Size", tag = "1")]
        pub size: i32,
        #[prost(bytes, tag = "2")]
        pub buffer: std::vec::Vec<u8>,
        #[prost(string, tag = "3")]
        pub file_extension: std::string::String,
        #[prost(string, tag = "4")]
        pub aspect_ratio: std::string::String,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UploadRequest {
        #[prost(bytes, tag = "1")]
        pub image: std::vec::Vec<u8>,
        #[prost(enumeration = "MimeType", tag = "2")]
        pub mimetype: i32,
        #[prost(string, tag = "3")]
        pub file_name: std::string::String,
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration,
    )]
    #[repr(i32)]
    pub enum Size {
        /// the ordinal one with compression
        Original = 0,
        /// VERY SMALL VARIANT LESS THAN 1 K 20X20
        Placeholder = 1,
        /// smaller variant  200x200 thumbnail
        Thumbnail = 2,
        /// smaller variant  400x4æ00 thumbnail
        Small = 3,
        /// small variant fo the image 500*500
        Medium = 4,
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration,
    )]
    #[repr(i32)]
    pub enum AspectRatio {
        Default = 0,
        X16x9 = 1,
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration,
    )]
    #[repr(i32)]
    pub enum MimeType {
        Png = 0,
        Jpeg = 1,
        Gif = 2,
        Webp = 3,
    }
}
/// Generated server implementations.
pub mod media_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for
    /// use with MediaServer.
    #[async_trait]
    pub trait Media: Send + Sync + 'static {
        /// Server streaming response type for the Upload method.
        type UploadStream: Stream<Item = Result<super::media::UploadResponse, tonic::Status>>
            + Send
            + Sync
            + 'static;
        async fn upload(
            &self,
            request: tonic::Request<super::media::UploadRequest>,
        ) -> Result<tonic::Response<Self::UploadStream>, tonic::Status>;
        async fn upload_and_write(
            &self,
            request: tonic::Request<super::media::UploadRequest>,
        ) -> Result<
            tonic::Response<super::media::UploadAndWriteResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct MediaServer<T: Media> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: Media> MediaServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }

        pub fn with_interceptor(
            inner: T,
            interceptor: impl Into<tonic::Interceptor>,
        ) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T: Media> Service<http::Request<HyperBody>> for MediaServer<T> {
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        type Response = http::Response<tonic::body::BoxBody>;

        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: http::Request<HyperBody>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/atwany.Media/Upload" => {
                    struct UploadSvc<T: Media>(pub Arc<T>);
                    impl<T: Media>
                        tonic::server::ServerStreamingService<
                            super::media::UploadRequest,
                        > for UploadSvc<T>
                    {
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        type Response = super::media::UploadResponse;
                        type ResponseStream = T::UploadStream;

                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::media::UploadRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { inner.upload(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1;
                        let inner = inner.0;
                        let method = UploadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(
                                codec,
                                interceptor,
                            )
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                },
                "/atwany.Media/UploadAndWrite" => {
                    struct UploadAndWriteSvc<T: Media>(pub Arc<T>);
                    impl<T: Media>
                        tonic::server::UnaryService<super::media::UploadRequest>
                        for UploadAndWriteSvc<T>
                    {
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        type Response = super::media::UploadAndWriteResponse;

                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::media::UploadRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                inner.upload_and_write(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = UploadAndWriteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(
                                codec,
                                interceptor,
                            )
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                },
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Media> Clone for MediaServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: Media> Clone for _Inner<T> {
        fn clone(&self) -> Self { Self(self.0.clone(), self.1.clone()) }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Media> tonic::transport::NamedService for MediaServer<T> {
        const NAME: &'static str = "atwany.Media";
    }
}
