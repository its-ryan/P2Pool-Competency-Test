use tower::Service;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TowerService;

impl Service<Vec<u8>> for TowerService {
    type Response = Vec<u8>;
    // Include 'static so the error can be boxed.
    type Error = Box<dyn Error + Send + Sync + 'static>;
    // Box the future so that its error type is sized.
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        // Example processing: simply echo back the incoming message.
        Box::pin(async move { Ok(req) })
    }
}
