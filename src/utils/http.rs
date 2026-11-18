use anyhow::Result;
use cookie::{
    Cookie,
    prefix::{
        Host,
        Prefix,
    },
};
use http::HeaderMap;

// Functions
pub(crate) fn extract_host_cookie_from_request_headers(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    Cookie::split_parse_encoded(cookie_str)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name().strip_prefix(Host::PREFIX) == Some(cookie_name))
        .map(|cookie| cookie.value().to_owned())
}

pub(crate) fn extract_client_ip_from_request_headers(headers: &HeaderMap) -> Option<&str> {
    let forwarded_for = headers.get("x-forwarded-for")?.to_str().ok()?;
    let client_ip = forwarded_for.split(',').next()?.trim();
    (!client_ip.is_empty()).then_some(client_ip)
}
