/// Stable continuation key for a sorted segment scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageToken {
    pub after_key: String,
    pub page_size: usize,
}

/// Return the next key boundary when the page was full.
pub fn next_page(last_key: &str, returned: usize, token: &PageToken) -> Option<String> {
    (returned >= token.page_size).then(|| last_key.to_owned())
}
