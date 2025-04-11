use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

#[derive(Clone)]
pub struct TimeService;

impl Service<Vec<u8>> for TimeService {
    type Response = Vec<u8>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        let res = [b"Echo: ".to_vec(), req].concat();
        Box::pin(async move { Ok(res) })
    }
}