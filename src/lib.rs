extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex as _regex;

pub mod regex;

use std::io;
use std::sync::Arc;
use hyper::server::{Service, NewService, Request, Response};
use hyper::{Method, StatusCode, Error as HyperError};
use futures::{future, Future};
use futures::future::BoxFuture;


pub trait RouteHandler<C>: 'static + Send + Sync {
    fn handle(&self, req: Request, cap: C) -> BoxFuture<Response, HyperError>;
}

impl<F, C> RouteHandler<C> for F
where
    F: 'static
        + Send
        + Sync
        + Fn(Request, C) -> BoxFuture<Response, HyperError>,
{
    fn handle(&self, req: Request, cap: C) -> BoxFuture<Response, HyperError> {
        (*self)(req, cap)
    }
}



pub trait RouteRecognizer {
    type Captures: 'static;
    fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(&RouteHandler<Self::Captures>, Self::Captures), StatusCode>;
}


pub trait RoutesBuilder: Sized {
    type Recognizer: RouteRecognizer;

    /// Add a new route with given glob pattern.
    fn route<S, H>(self, method: Method, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>;

    /// Create recoginizer
    fn finish(self) -> Self::Recognizer;

    /// Add handler for 'GET' method
    fn get<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Get, pattern, handler)
    }

    /// Add handler for 'POST' method
    fn post<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Post, pattern, handler)
    }

    /// Add handler for 'PUT' method
    fn put<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Put, pattern, handler)
    }

    /// Add handler for 'DELETE' method
    fn delete<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Delete, pattern, handler)
    }

    /// Add handler for 'HEAD' method
    fn head<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Head, pattern, handler)
    }

    /// Add handler for 'OPTIONS' method
    fn options<S, H>(self, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler<<Self::Recognizer as RouteRecognizer>::Captures>,
    {
        self.route(Method::Options, pattern, handler)
    }
}



pub struct Router<R: RouteRecognizer> {
    inner: Arc<R>,
}

impl<R: RouteRecognizer> From<R> for Router<R> {
    fn from(recognizer: R) -> Self {
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
        let method = req.method().clone();
        let path = req.path().to_owned();
        match self.inner.recognize(&method, &path) {
            Ok((handler, cap)) => handler.handle(req, cap),
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
    }
}
