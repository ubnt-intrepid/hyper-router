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
    fn find_handler(
        &self,
        path: &str,
        method: &Method,
    ) -> Result<(&Box<RouteHandler>, Vec<String>), StatusCode> {
        let mut get_route = None;
        let mut has_other_method = false;
        for route in &self.routes {
            if route.method == *method {
                if let Some(cap) = get_owned_captures(&route.pattern, path) {
                    return Ok((&route.handler, cap));
                }
            } else {
                if let Some(cap) = get_owned_captures(&route.pattern, path) {
                    // different method, but matched pattern
                    has_other_method = true;
                    if route.method == Method::Get {
                        if get_route.is_none() {
                            get_route = Some((&route.handler, cap));
                        }
                    }
                }
            }
        }
        let err_code = if has_other_method {
            StatusCode::MethodNotAllowed
        } else {
            StatusCode::NotFound
        };

        if *method == Method::Head {
            get_route.ok_or(err_code)
        } else {
            Err(err_code)
        }
    }
}


fn get_owned_captures(re: &Regex, path: &str) -> Option<Vec<String>> {
    re.captures(path).map(|cap| {
        cap.iter()
            .skip(1)
            .map(|s| s.unwrap().as_str().to_owned())
            .collect()
    })
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
        match self.inner.find_handler(
            req.path(),
            req.method(),
        ) {
            Ok((handler, cap)) => handler.handle(req, cap),
            Err(code) => future::ok(Response::new().with_status(code)).boxed(),
        }
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
