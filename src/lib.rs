extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex as _regex;

pub mod regex;
pub use regex::{RegexRoutesBuilder, RegexRouteRecognizer};

use std::io;
use std::sync::Arc;
use hyper::server::{Service, NewService, Request, Response};
use hyper::{Method, StatusCode, Error as HyperError};
use futures::{future, Future};
use futures::future::BoxFuture;


pub trait RouteHandler: 'static + Send + Sync {
    fn handle(&self, req: Request, cap: Vec<String>) -> BoxFuture<Response, HyperError>;
}

impl<F> RouteHandler for F
where
    F: 'static
        + Send
        + Sync
        + Fn(Request, Vec<String>) -> BoxFuture<Response, HyperError>,
{
    fn handle(&self, req: Request, cap: Vec<String>) -> BoxFuture<Response, HyperError> {
        (*self)(req, cap)
    }
}



pub trait RouteRecognizer {
    fn find_handler(
        &self,
        path: &str,
        method: &Method,
    ) -> Result<(&RouteHandler, Vec<String>), StatusCode>;
}


pub trait RoutesBuilder: Sized {
    type Recognizer: RouteRecognizer;

    /// Add a new route with given glob pattern.
    fn route<S, H>(self, method: Method, pattern: S, handler: H) -> Self
    where
        S: Into<String>,
        H: RouteHandler;

    /// Create recoginizer
    fn finish(self) -> Self::Recognizer;

    fn get<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Get, pattern, handler)
    }

    fn post<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Post, pattern, handler)
    }

    fn put<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Put, pattern, handler)
    }

    fn delete<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Delete, pattern, handler)
    }

    fn head<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Head, pattern, handler)
    }

    fn options<S: Into<String>, H: RouteHandler>(self, pattern: S, handler: H) -> Self {
        self.route(Method::Options, pattern, handler)
    }
}



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
