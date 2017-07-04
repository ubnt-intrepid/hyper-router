extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;

pub mod router;
#[path = "regex.rs"]
pub mod regex_router;

pub use router::{Router, RouterService};
pub use regex_router::{RegexRoutesBuilder, RegexRouteRecognizer};


use hyper::server::{Request, Response};
use hyper::{Method, StatusCode, Error as HyperError};
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
    ) -> Result<(&Box<RouteHandler>, Vec<String>), StatusCode>;
}
