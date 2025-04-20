use tower::Service;
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use std::error::Error;

// Defines a simple service that echoes back received byte vectors.
#[derive(Debug, Clone)]
pub struct TowerService;

// Implements the Service trait from the tower crate for TowerService.
// This allows TowerService to act as a middleware or application logic component
// within a tower-based architecture.
impl Service<Vec<u8>> for TowerService {
    // Defines the successful response type of the service as a vector of bytes.
    type Response = Vec<u8>;
    // Defines the error type of the service. It's boxed to allow for various
    // concrete error types and requires Send, Sync, and 'static for thread safety
    // and lifetime independence.
    type Error = Box<dyn Error + Send + Sync + 'static>;
    // Defines the future returned by the `call` method. It's pinned and boxed
    // to ensure it's movable across await points and has a concrete, sized type.
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    // Indicates whether the service is ready to process a new request.
    // In this simple example, the service is always ready.
    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // Processes an incoming request of type `Vec<u8>` and returns a `Future`
    // that will eventually yield a `Result` containing either the response
    // (`Vec<u8>`) or an error (`Self::Error`).
    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        // This asynchronous block defines the operation performed when the service
        // is called. In this case, it simply returns the input request as the response,
        // effectively echoing the received message. The `async move` keyword ensures
        // that the closure captures the `req` by value and can be moved across
        // await points. `Box::pin` converts the `Future` returned by the `async`
        // block into a pinned and boxed `Future` as required by the `Service` trait.
        Box::pin(async move { Ok(req) })
    }
}