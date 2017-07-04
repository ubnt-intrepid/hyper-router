use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use hyper::server::{Service, NewService, Request, Response};
use hyper::Error as HyperError;
use hyper::{Method, StatusCode};
use futures::Future;
use futures::future::{self, BoxFuture};
use regex::Regex;


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


/// Builder object of Router;
#[derive(Default)]
pub struct RouteBuilder<'a> {
    routes: HashMap<(Cow<'a, str>, Method), Box<RouteHandler>>,
}

impl<'a> RouteBuilder<'a> {
    /// Add a new route with given glob pattern.
    pub fn route<'b: 'a, H: RouteHandler>(
        mut self,
        method: Method,
        pattern: &'b str,
        handler: H,
    ) -> Self {
        let pattern = normalize_pattern(pattern);
        let handler = Box::new(handler);
        self.routes.insert(
            (pattern, method),
            handler,
        );
        self
    }

    pub fn get<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Get, pattern, handler)
    }

    pub fn post<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Post, pattern, handler)
    }

    /// Finalize building router.
    pub fn finish(self) -> Router {
        let inner = Arc::new(RouterInner {
            routes: self.routes
                .into_iter()
                .map(|((pattern, method), handler)| {
                    Route {
                        pattern: Regex::new(&pattern).unwrap(),
                        method,
                        handler,
                    }
                })
                .collect(),
        });
        Router { inner }
    }
}


struct Route {
    pattern: Regex,
    method: Method,
    handler: Box<RouteHandler>,
}

#[derive(Default)]
struct RouterInner {
    routes: Vec<Route>,
}

impl RouterInner {
    fn find_matches(&self, path: &str) -> Vec<(&Route, Vec<String>)> {
        self.routes
            .iter()
            .filter_map(|route| {
                route.pattern.captures(path).map(|cap| {
                    let cap = cap.iter()
                        .skip(1)
                        .map(|s| s.unwrap().as_str().to_owned())
                        .collect();
                    (route, cap)
                })
            })
            .collect()
    }
}

pub struct Router {
    inner: Arc<RouterInner>,
}

impl NewService for Router {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Instance = RouterService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(RouterService { inner: self.inner.clone() })
    }
}


/// An asynchronous task executed by hyper.
pub struct RouterService {
    inner: Arc<RouterInner>,
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Future = BoxFuture<Response, HyperError>;

    fn call(&self, req: Request) -> Self::Future {
        let matches = self.inner.find_matches(req.path());
        if matches.len() == 0 {
            return future::ok(Response::new().with_status(StatusCode::NotFound)).boxed();
        }
        for (route, cap) in matches {
            if route.method == *req.method() {
                return route.handler.handle(req, cap);
            }
        }
        future::ok(Response::new().with_status(StatusCode::MethodNotAllowed)).boxed()
    }
}


fn normalize_pattern(pattern: &str) -> Cow<str> {
    let pattern = pattern
        .trim()
        .trim_left_matches("^")
        .trim_right_matches("$")
        .trim_right_matches("/");
    match pattern {
        "" => "^/$".into(),
        s => format!("^{}/?$", s).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_pattern;

    #[test]
    fn normalize_cases() {
        assert_eq!(normalize_pattern("/"), "^/$");
        assert_eq!(normalize_pattern("/path/to"), "^/path/to/?$");
        assert_eq!(normalize_pattern("/path/to/"), "^/path/to/?$");
    }
}
