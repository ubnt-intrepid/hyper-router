extern crate hyper;
extern crate hyper_router;
extern crate futures;
extern crate regex;

use hyper::server::{Http, Request, Response};
use hyper::{Get, Post, StatusCode};
use hyper::Error as HyperError;
use hyper_router::RouteBuilder;
use futures::{Future, Stream};
use futures::future::{self, BoxFuture};

fn index(_req: Request, _cap: Vec<String>) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn index_post(req: Request, _cap: Vec<String>) -> BoxFuture<Response, HyperError> {
    req.body()
        .collect()
        .and_then(|chunks| {
            let mut body = Vec::new();
            for chunk in chunks {
                body.extend_from_slice(&chunk);
            }
            future::ok(
                Response::new()
                    .with_status(StatusCode::Ok)
                    .with_body(format!("Posted: {}", String::from_utf8_lossy(&body))),
            )
        })
        .boxed()
}

fn show_captures(_req: Request, cap: Vec<String>) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", cap)),
    ).boxed()
}

fn main() {
    let router = RouteBuilder::default()
        .route(Get, "/", index)
        .route(Post, "/", index_post)
        .route(Get, r"/([^/]+)", show_captures)
        .finish();

    let addr = "0.0.0.0:4000".parse().unwrap();
    let server = Http::new().bind(&addr, router).unwrap();
    server.run().unwrap();
}
