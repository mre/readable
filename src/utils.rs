use axum::{
    extract::TypedHeader,
    headers::UserAgent
};
use reqwest::header::HeaderValue;


const DEFAULT_USER_AGENT: HeaderValue = HeaderValue::from_static(
    concat!("Readable/", env!("CARGO_PKG_VERSION"))
);


pub fn forwarded_agent(ua_header: &Option<TypedHeader<UserAgent>>) -> HeaderValue {
    match ua_header {
        Some(TypedHeader(ua)) => {
            HeaderValue::from_str(ua.as_str()).unwrap_or(DEFAULT_USER_AGENT)
        },
        None => DEFAULT_USER_AGENT,
    }
}


/// get current date and time as UTC
/// and format as: 1 December, 2017 12:00:00
pub fn get_time() -> String {
    let now = chrono::Local::now();
    now.format("%A, %B %e, %Y, %H:%M:%S").to_string()
}
