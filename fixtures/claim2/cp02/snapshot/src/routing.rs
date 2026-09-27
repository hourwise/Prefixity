/// One route from a key interval to an immutable segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRoute {
    pub first_key: String,
    pub last_key: String,
    pub segment: String,
}

/// Select the first range containing a key, preserving manifest order.
pub fn route_key<'a>(routes: &'a [KeyRoute], key: &str) -> Option<&'a KeyRoute> {
    routes
        .iter()
        .find(|route| key >= route.first_key.as_str() && key <= route.last_key.as_str())
}
