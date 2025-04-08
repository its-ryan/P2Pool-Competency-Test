use tower::{Service, ServiceExt};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use crate::custom_protocol::{TimeRequest, TimeResponse};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct TimeService;

impl Service<TimeRequest> for TimeService {
    type Response = TimeResponse;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<TimeResponse, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: TimeRequest) -> Self::Future {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let response = TimeResponse(format!("Unix Time: {}", now.as_secs()).into_bytes());
        Box::pin(async move { Ok(response) })
    }
}
