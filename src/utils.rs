use anyhow::{Context, Result};
use reqwest::Url;

pub fn check_base_url(url: &str) -> Result<String> {
    let new_url = Url::parse(url).with_context(|| format!("Base URL invalid"))?;
    Ok(new_url.into_string())
}

pub fn normalise_url(base_url: &str, url: &str) -> Option<String> {
    let new_base_url = Url::parse(base_url);
    let new_url = Url::parse(url);

    if new_base_url.is_ok() {
        let nbu = new_base_url.unwrap();
        match new_url {
            Ok(new_url) => {
                // Only allow URLs from domain.
                // Ignore URLS with ? for get requests and # for client side frameworks.
                if nbu.to_owned().host_str() == new_url.host_str()
                    && !url.contains("?")
                    && !url.contains("#")
                {
                    Some(url.to_owned())
                } else {
                    None
                }
            }
            Err(_e) => {
                // Relative urls are not parsed by Reqwest.
                // Ignore URLS with ? for get requests and # for client side frameworks.
                if url.starts_with('/') && !url.contains("?") && !url.contains("#") {
                    let scheme = nbu.scheme();
                    let host = nbu.host_str().unwrap();
                    Some(format!("{}://{}{}", scheme, host, url))
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}
