//!
//! Defines Regex router
//!

use std::borrow::Cow;
use std::collections::HashMap;
use _regex::Regex;
use hyper::{Method, StatusCode};
use super::{RouteHandler, RouteRecognizer, RoutesBuilder};


struct RegexRoute {
    pattern: Regex,
    handler: Box<RouteHandler>,
}


/// Builder for RegexRouteRecognizer.
#[derive(Default)]
pub struct RegexRoutesBuilder {
    routes: HashMap<Method, Vec<RegexRoute>>,
}

impl RoutesBuilder for RegexRoutesBuilder {
    type Recognizer = RegexRouteRecognizer;

    fn route<S, H>(mut self, method: Method, pattern: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: RouteHandler,
    {
        let pattern = normalize_pattern(pattern.as_ref());
        let pattern = Regex::new(&pattern).unwrap();
        let handler = Box::new(handler);
        self.routes
            .entry(method)
            .or_insert(Vec::new())
            .push(RegexRoute { pattern, handler });
        self
    }

    fn finish(self) -> Self::Recognizer {
        RegexRouteRecognizer { routes: self.routes }
    }
}


pub struct RegexRouteRecognizer {
    routes: HashMap<Method, Vec<RegexRoute>>,
}

impl RouteRecognizer for RegexRouteRecognizer {
    fn recognize(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<(&RouteHandler, Vec<String>), StatusCode> {
        let routes = self.routes.get(method).ok_or(
            StatusCode::NotFound,
        )?;
        for route in routes {
            if let Some(caps) = get_owned_captures(&route.pattern, path) {
                return Ok((&*route.handler, caps));
            }
        }
        Err(StatusCode::NotFound)
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
