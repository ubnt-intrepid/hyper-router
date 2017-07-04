# `hyper-router`
[![Build Status](https://travis-ci.org/ubnt-intrepid/hyper-router.svg?branch=master)](https://travis-ci.org/ubnt-intrepid/hyper-router)

An alternative of router middleware for Hyper 0.11

## Usage
See also [simple.rs](examples/simple.rs).

```rust
extern crate hyper;
extern crate hyper_router;
extern crate futures;

use hyper::server::{Http, Request, Response};
use hyper::{Get, Post, StatusCode, Error as HyperError};
use hyper_router::RouteBuilder;
use futures::{future, Future};
use futures::future::BoxFuture;

fn index(_req: Request, _cap: Vec<String>) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body("Hello, world"),
    ).boxed()
}

fn post_index(_req: Request, cap: Vec<String>) -> BoxFuture<Response, HyperError> {
    future::ok(
        Response::new()
            .with_status(StatusCode::Ok)
            .with_body(format!("Captures: {:?}", cap)),
    ).boxed()
}

fn main() {
    let router = RouteBuilder::default()
        .get("/", index)
        .post(r"/([^/]+)", post_index)
        .finish();

    let addr = "0.0.0.0:4000".parse().unwrap();
    let server = Http::new().bind(&addr, router).unwrap();
    server.run().unwrap();
}
```

```toml
[dependencies]
hyper = "~0.11"
hyper-router = { git = "https://github.com/ubnt-intrepid/hyper-router.git" }
futures = "0.1"
```

