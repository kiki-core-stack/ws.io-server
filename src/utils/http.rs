use std::{
    env,
    net::IpAddr,
    sync::LazyLock,
};

use anyhow::Result;
use cookie::{
    Cookie,
    prefix::{
        Host,
        Prefix,
    },
};
use http::HeaderMap;
use ipnet::IpNet;

// Constants/Statics
static TRUSTED_PROXIES: LazyLock<Vec<IpNet>> = LazyLock::new(|| {
    let value = env::var("SERVER_TRUSTED_PROXIES").unwrap_or_else(|_| "loopback,linklocal,uniquelocal".to_owned());
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .flat_map(|entry| match entry {
            "linklocal" => vec!["169.254.0.0/16", "fe80::/10"],
            "loopback" => vec!["127.0.0.0/8", "::1/128"],
            "uniquelocal" => vec!["10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16", "fc00::/7"],
            _ => vec![entry],
        })
        .map(|subnet| {
            subnet
                .parse::<IpNet>()
                .or_else(|_| subnet.parse::<IpAddr>().map(IpNet::from))
                .unwrap_or_else(|_| panic!("Invalid SERVER_TRUSTED_PROXIES entry: {subnet}"))
        })
        .collect()
});

// Functions
pub(crate) fn extract_host_cookie_from_request_headers(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    Cookie::split_parse_encoded(cookie_str)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name().strip_prefix(Host::PREFIX) == Some(cookie_name))
        .map(|cookie| cookie.value().to_owned())
}

/// Resolves the client IP like the backends' `getClientIp`: each trusted hop is
/// replaced by the next `X-Forwarded-For` entry from the right, and the first
/// untrusted address is the client. The connection peer is not available here,
/// so the proxy connecting to this service (the gateway) is always trusted.
pub(crate) fn extract_client_ip_from_request_headers(headers: &HeaderMap) -> Option<IpAddr> {
    let forwarded_for = headers.get("x-forwarded-for")?.to_str().ok()?;
    let mut client_ip = None;
    for entry in forwarded_for.rsplit(',') {
        let Ok(ip) = entry.trim().parse::<IpAddr>() else { break };
        let ip = ip.to_canonical();
        client_ip = Some(ip);
        if !TRUSTED_PROXIES.iter().any(|subnet| subnet.contains(&ip)) {
            break;
        }
    }

    client_ip
}
