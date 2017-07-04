use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use hyper::server::{Service, NewService, Request, Response};
use hyper::Error as HyperError;
use hyper::Method;
use futures::Future;
use futures::future::{self, BoxFuture};

use super::{RouteHandler, RouteRecognizer};


/// Builder for Router.
#[derive(Default)]
pub struct RoutesBuilder<'a> {
    routes: HashMap<(Cow<'a, str>, Method), Box<RouteHandler>>,
}

impl<'a> RoutesBuilder<'a> {
    /// Add a new route with given glob pattern.
    pub fn route<'b: 'a, H: RouteHandler>(
        mut self,
        method: Method,
        pattern: &'b str,
        handler: H,
    ) -> Self {
        let handler = Box::new(handler);
        self.routes.insert(
            (pattern.into(), method),
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

    pub fn put<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Put, pattern, handler)
    }

    pub fn delete<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Delete, pattern, handler)
    }

    pub fn head<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Head, pattern, handler)
    }

    pub fn options<'b: 'a, H: RouteHandler>(self, pattern: &'b str, handler: H) -> Self {
        self.route(Method::Options, pattern, handler)
    }

    /// Finalize building router.
    pub fn finish<R>(self) -> Router<R>
    where
        for<'t> R: RouteRecognizer + From<HashMap<(Cow<'t, str>, Method), Box<RouteHandler>>>,
    {
        Router::<R>::new(self.routes)
    }
}


pub struct Router<R: RouteRecognizer> {
    inner: Arc<R>,
}

impl<R> Router<R>
where
    for<'t> R: RouteRecognizer
               + From<HashMap<(Cow<'t, str>, Method), Box<RouteHandler>>>,
{
    fn new(routes: HashMap<(Cow<str>, Method), Box<RouteHandler>>) -> Self {
        let inner = Arc::new(R::from(routes));
        Router { inner }
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
