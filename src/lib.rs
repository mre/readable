use axum::{
    body::Body,
    extract::TypedHeader,
    headers::UserAgent,
    http::{HeaderValue, StatusCode, Uri},
    response::{self, Html, IntoResponse, Response},
    routing::get,
    Router,
};
use readable_readability::Readability;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use sync_wrapper::SyncWrapper;

mod utils;


pub fn index() -> Html<String> {
    render(
        "Readable.",
        "Readable",
        "A simple web service to extract the main content from an article<br /> and format it for <i>reading</i>.
        Source code <a href=\"https://github.com/mre/readable\">here</a>.
        ",
        include_str!("../static/index.html"),
        None,
    )
}

pub async fn readable(url: Uri, ua_header: Option<TypedHeader<UserAgent>>) -> Result<impl IntoResponse, (StatusCode, Html<String>)> {
    // Strip the leading slash. Not sure if there's a better way to do this.
    let path = url.path().trim_start_matches('/');

    if path.is_empty() {
        // This could probably be moved into the router
        return Ok(index());
    }

    // Convert to `url::Url`. This is needed later but it also validates the URL.
    let url = url::Url::parse(path).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            render(
                "Invalid URL",
                "Invalid URL",
                "Check if the path represents a valid URL",
                &e.to_string(),
                None,
            ),
        )
    })?;

    let body = reqwest::Client::new()
        .get(url.clone())
        .header(USER_AGENT, utils::forwarded_agent(&ua_header))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                render(
                    "Yikes!",
                    "Yikes!",
                    "Couldn't render article. (It is an article, right?)",
                    &format!("Can't fetch URL: {e}"),
                    None,
                ),
            )
        })?
        .text()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                render(
                    "Yikes!",
                    "Yikes!",
                    "Couldn't render article. (It is an article, right?)",
                    &format!("Can't fetch response body text: {e}"),
                    None,
                ),
            )
        })?;

    let (content_root, meta) = Readability::new().base_url(Some(url.clone())).parse(&body);
    let mut content_bytes = vec![];
    content_root.serialize(&mut content_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            render(
                "Ouch",
                "Ouch",
                "Couldn't extract content form the article.(It is an article, right?)",
                &format!("Can't serialize content: {e}"),
                None,
            ),
        )
    })?;
    let content = std::str::from_utf8(&content_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            render(
                "Humm...",
                "Humm...",
                "Invalid UTF-8 in article content",
                &format!("Can't serialize content: {e}"),
                None,
            ),
        )
    })?;

    let header = format!(
        "A readable version of <a class=\"shortened\" href={url}>{url}</a><br />retrieved on {}",
        utils::get_time()
    );
    Ok(render(
        &meta.page_title.unwrap_or_else(|| "Readable".into()),
        &meta.article_title.unwrap_or_else(|| "Readable".into()),
        &header,
        content,
        Some(url.as_str()),
    ))
}

fn render(
    page_title: &str,
    article_title: &str,
    header: &str,
    content: &str,
    canonical: Option<&str>,
) -> Html<String> {
    let template = include_str!("../static/template.html");
    let mut output = template
        .replace("{{page_title}}", page_title)
        .replace("{{article_title}}", article_title)
        .replace("{{header}}", header)
        .replace("{{content}}", content);

    if let Some(canonical) = canonical {
        output = output.replace(
            "{{canonical}}",
            &format!("<link rel=\"canonical\" href=\"{canonical}\" />"),
        );
    } else {
        output = output.replace("{{canonical}}", "");
    }

    response::Html(output)
}

pub fn static_content(
    content: &'static [u8],
    content_type: HeaderValue,
) -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(content.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route(
            "/static/Crimson.woff2",
            get(|| async {
                static_content(
                    include_bytes!("../static/fonts/Crimson.woff2",),
                    HeaderValue::from_static("text/woff2"),
                )
            }),
        )
        .route(
            "/static/JetBrainsMono.woff2",
            get(|| async {
                static_content(
                    include_bytes!("../static/fonts/JetBrainsMono.woff2",),
                    HeaderValue::from_static("font/woff2"),
                )
            }),
        )
        .fallback(readable);
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
