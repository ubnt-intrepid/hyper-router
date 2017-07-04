use std::io;
use std::sync::Arc;

use hyper::server::{Service, NewService, Request, Response};
use hyper::Error as HyperError;
use futures::Future;
use futures::future::{self, BoxFuture};

use super::RouteRecognizer;


pub struct Router<R: RouteRecognizer> {
    inner: Arc<R>,
}

impl<R: RouteRecognizer> Router<R> {
    pub fn new(recognizer: R) -> Self {
        Router { inner: Arc::new(recognizer) }
    }
}

impl<R> NewService for Router<R>
where
    R: RouteRecognizer,
{
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Instance = RouterService<R>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(RouterService { inner: self.inner.clone() })
    }
}


/// An asynchronous task executed by hyper.
pub struct RouterService<R: RouteRecognizer> {
    inner: Arc<R>,
}

impl<R> Service for RouterService<R>
where
    R: RouteRecognizer,
{
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Future = BoxFuture<Response, HyperError>;

    fn call(&self, req: Request) -> Self::Future {
        match self.inner.find_handler(
            req.path(),
            req.method(),
        ) {
            Ok((handler, cap)) => handler.handle(req, cap),
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
    }
}
