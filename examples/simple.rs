extern crate hyper;
extern crate hyper_router;
extern crate futures;
extern crate regex;

use hyper::server::{Http, Request, Response};
use hyper::{Get, Post, StatusCode};
use hyper::Error as HyperError;
use hyper_router::RouteBuilder;
use futures::Future;
use futures::future::{self, BoxFuture};
use regex::Captures;

fn index(_req: &Request, _cap: &Captures) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn post_index(_req: &Request, cap: &Captures) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", cap)),
    ).boxed()
}

fn main() {
    let router = RouteBuilder::default()
        .route(Get, "/", index)
        .route(Post, r"/([^/]+)", post_index)
        .finish();

    let addr = "0.0.0.0:4000".parse().unwrap();
    let server = Http::new().bind(&addr, router).unwrap();
    server.run().unwrap();
}
