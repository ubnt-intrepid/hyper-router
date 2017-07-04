use std::borrow::Cow;
use std::collections::HashMap;
use regex::Regex;
use hyper::{Method, StatusCode};
use super::{RouteHandler, RouteRecognizer};

struct RegexRoute {
    pattern: Regex,
    method: Method,
    handler: Box<RouteHandler>,
}

#[derive(Default)]
pub struct RegexRouteRecognizer {
    routes: Vec<RegexRoute>,
}

impl<'t> From<HashMap<(Cow<'t, str>, Method), Box<RouteHandler>>> for RegexRouteRecognizer {
    fn from(routes: HashMap<(Cow<'t, str>, Method), Box<RouteHandler>>) -> Self {
        RegexRouteRecognizer {
            routes: routes
                .into_iter()
                .map(|((pattern, method), handler)| {
                    let pattern = normalize_pattern(&pattern);
                    RegexRoute {
                        pattern: Regex::new(&pattern).unwrap(),
                        method,
                        handler,
                    }
                })
                .collect(),
        }
    }
}

impl RouteRecognizer for RegexRouteRecognizer {
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
