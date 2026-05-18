use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use reqwest::{Client, Proxy};
use std::net::SocketAddr;
use std::{collections::HashMap, io::Result};
use tokio::net::TcpListener;
use tracing::{error, info};
use url::Url;

// 定义命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 服务器运行的端口
    #[arg(short, long, default_value_t = 40211)]
    port: u16,
}

fn should_skip_request_header(header_name: &str) -> bool {
    matches!(
        header_name,
        "accept-encoding"
            | "connection"
            | "content-length"
            | "host"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "proxy-connection"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn build_forward_request_headers(headers: &HeaderMap) -> HeaderMap {
    let mut forwarded_headers = HeaderMap::new();

    for (name, value) in headers {
        if should_skip_request_header(name.as_str()) {
            continue;
        }

        forwarded_headers.append(name.clone(), value.clone());
    }

    forwarded_headers
}

async fn forward_request(
    request_headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let url_param = params.get("url");
    let proxy_param = params.get("proxy");

    let url_str = match url_param {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Missing 'url' parameter").into_response(),
    };

    // 解码 URL
    let decoded_url_str = match urlencoding::decode(url_str) {
        Ok(s) => s.to_string(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to decode URL parameter",
            )
                .into_response();
        }
    };

    // 验证并解析 URL
    let target_url = match Url::parse(&decoded_url_str) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid URL: {}", e)).into_response(),
    };

    let mut client_builder = Client::builder();

    if let Some(proxy_str) = proxy_param {
        // 解码代理 URL
        let decoded_proxy_str = match urlencoding::decode(proxy_str) {
            Ok(s) => s.to_string(),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to decode proxy parameter",
                )
                    .into_response();
            }
        };

        match Proxy::all(&decoded_proxy_str) {
            Ok(proxy) => {
                client_builder = client_builder.proxy(proxy);
            }
            Err(e) => {
                return (StatusCode::BAD_REQUEST, format!("Invalid proxy URL: {}", e))
                    .into_response();
            }
        }
    }

    let client = match client_builder.build() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to build reqwest client: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build HTTP client: {}", e),
            )
                .into_response();
        }
    };

    let forwarded_headers = build_forward_request_headers(&request_headers);

    match client.get(target_url).headers(forwarded_headers).send().await {
        Ok(response) => {
            let status = response.status();
            let mut headers = HeaderMap::new();

            // 转发 Content-Type 头
            if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
                if let Ok(ct_str) = content_type.to_str()
                    && let Ok(header_value) = ct_str.parse()
                {
                    headers.insert(axum::http::header::CONTENT_TYPE, header_value);
                }
            } else {
                // 如果没有Content-Type，默认为text/html
                if let Ok(header_value) = "text/html".parse() {
                    headers.insert(axum::http::header::CONTENT_TYPE, header_value);
                }
            }

            let body = match response.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read response body: {}", e),
                    )
                        .into_response();
                }
            };

            (status, headers, body).into_response()
        }
        Err(e) => {
            error!("Request to {} failed: {}", decoded_url_str, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Request failed: {}", e),
            )
                .into_response()
        }
    }
}

#[tokio::main] // axum 基于 tokio 运行时
async fn main() -> Result<()> {
    let args = Args::parse();
    let port = args.port;

    // 初始化日志系统
    tracing_subscriber::fmt().init();

    info!("Starting Forward URL Proxy on http://localhost:{}", port);
    info!(
        "Example usage: http://localhost:{}?url=https%3A%2F%2Fgoogle.com&proxy=http%3A%2F%2Flocalhost%3A7890",
        port
    );

    let app = Router::new().route("/", get(forward_request));

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    info!("listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|e| std::io::Error::other(e.to_string())) // 转换为 std::io::Error
}

#[cfg(test)]
mod tests {
    use super::build_forward_request_headers;
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn forwards_regular_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer token"));
        headers.insert("x-test-header", HeaderValue::from_static("value"));

        let forwarded_headers = build_forward_request_headers(&headers);

        assert_eq!(
            forwarded_headers.get(header::AUTHORIZATION),
            Some(&HeaderValue::from_static("Bearer token"))
        );
        assert_eq!(
            forwarded_headers.get("x-test-header"),
            Some(&HeaderValue::from_static("value"))
        );
    }

    #[test]
    fn skips_proxy_sensitive_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("localhost:40211"));
        headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert("proxy-authorization", HeaderValue::from_static("secret"));
        headers.insert("x-test-header", HeaderValue::from_static("value"));

        let forwarded_headers = build_forward_request_headers(&headers);

        assert!(forwarded_headers.get(header::HOST).is_none());
        assert!(forwarded_headers.get(header::CONNECTION).is_none());
        assert!(forwarded_headers.get("proxy-authorization").is_none());
        assert_eq!(
            forwarded_headers.get("x-test-header"),
            Some(&HeaderValue::from_static("value"))
        );
    }

    #[test]
    fn preserves_multiple_header_values() {
        let mut headers = HeaderMap::new();
        headers.append(header::COOKIE, HeaderValue::from_static("a=1"));
        headers.append(header::COOKIE, HeaderValue::from_static("b=2"));

        let forwarded_headers = build_forward_request_headers(&headers);
        let cookie_values = forwarded_headers
            .get_all(header::COOKIE)
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(cookie_values.len(), 2);
        assert_eq!(cookie_values[0], HeaderValue::from_static("a=1"));
        assert_eq!(cookie_values[1], HeaderValue::from_static("b=2"));
    }
}
