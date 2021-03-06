extern crate hyper;
extern crate hyper_router;
extern crate futures;
extern crate regex;

use hyper::server::{Http, Request, Response};
use hyper::{Error as HyperError, StatusCode};
use hyper_router::{Router, RoutesBuilder};
use hyper_router::regex::{Captures, RegexRoutesBuilder};
use futures::{future, Future, Stream};
use futures::future::BoxFuture;

fn index(_req: Request, _cap: Captures) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn index_post(req: Request, _cap: Captures) -> BoxFuture<Response, HyperError> {
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

fn show_captures(_req: Request, cap: Captures) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", cap)),
    ).boxed()
}

fn main() {
    let recognizer = RegexRoutesBuilder::default()
        .get("/", index)
        .post("/", index_post)
        .post("/post", index_post)
        .get(r"/echo/([^/]+)/(?P<hoge>[^/]+)/([^/]+)", show_captures)
        .finish();
    let router = Router::from(recognizer);

    let addr = "0.0.0.0:4000".parse().unwrap();
    let server = Http::new().bind(&addr, router).unwrap();
    server.run().unwrap();
}
