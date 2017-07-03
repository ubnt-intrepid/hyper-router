use std::io;
use std::sync::Arc;
use hyper::server::{Service, NewService, Request, Response};
use hyper::Error as HyperError;
use hyper::{Method, StatusCode};
use futures::Future;
use futures::future::{self, BoxFuture};
use futures_cpupool::CpuPool;
use regex::{Regex, Captures};


#[derive(Default)]
pub struct RouteBuilder(RouterInner);

impl RouteBuilder {
    pub fn route<H: RouteHandler>(mut self, method: Method, path: &str, handler: H) -> Self {
        let route = Route {
            method,
            path: path.to_owned(),
            path_re: Regex::new(&format!("^{}$", path)).unwrap(),
            handler: Box::new(handler),
        };
        self.0.routes.push(route);
        self
    }

    pub fn finish(self) -> Router {
        Router::new(self.0)
    }
}


pub trait RouteHandler: 'static + Send + Sync {
    fn handle(&self, req: &Request, cap: &Captures) -> BoxFuture<Response, HyperError>;
}

impl<F> RouteHandler for F
where
    F: 'static
        + Send
        + Sync
        + Fn(&Request, &Captures) -> BoxFuture<Response, HyperError>,
{
    fn handle(&self, req: &Request, cap: &Captures) -> BoxFuture<Response, HyperError> {
        (*self)(req, cap)
    }
}


#[allow(dead_code)]
struct Route {
    path: String,
    path_re: Regex,
    method: Method,
    handler: Box<RouteHandler>,
}


#[derive(Default)]
struct RouterInner {
    routes: Vec<Route>,
}

pub struct Router {
    inner: Arc<RouterInner>,
    thread_pool: CpuPool,
}

impl Router {
    fn new(inner: RouterInner) -> Self {
        let inner = Arc::new(inner);
        let thread_pool = CpuPool::new_num_cpus();
        Router { inner, thread_pool }
    }
}

impl NewService for Router {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Instance = RouterService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(RouterService {
            inner: self.inner.clone(),
            thread_pool: self.thread_pool.clone(),
        })
    }
}



pub struct RouterService {
    inner: Arc<RouterInner>,
    thread_pool: CpuPool,
}

impl Service for RouterService {
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    type Future = BoxFuture<Response, HyperError>;

    fn call(&self, req: Request) -> Self::Future {
        let matched = self.inner
            .routes
            .iter()
            .filter_map(|route| {
                route.path_re.captures(req.path()).map(
                    |cap| (cap, route),
                )
            })
            .next();
        if let Some((cap, m)) = matched {
            if m.method == *req.method() {
                self.thread_pool
                    .spawn(m.handler.handle(&req, &cap))
                    .boxed()
            } else {
                future::ok(Response::new().with_status(StatusCode::MethodNotAllowed)).boxed()
            }
        } else {
            future::ok(Response::new().with_status(StatusCode::NotFound)).boxed()
        }
    }
}
