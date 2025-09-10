use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use clap::Parser;
use reqwest::{Client, Proxy};
use std::{collections::HashMap, io::Result, path::PathBuf, str::FromStr};
use std::{io::Write, net::SocketAddr};
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

// 定义一个AppState来持有共享的reqwest Client，尽管在这个简单的例子中，
// 每次请求构建一个新的Client也是可行的，但共享Client通常更高效。
#[derive(Clone)]
struct AppState {
    http_client: Client,
}

// 异步处理函数，用于转发请求
async fn forward_request(
    State(_state): State<AppState>,
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

    // 执行请求
    match client.get(target_url).send().await {
        Ok(response) => {
            let status = response.status();
            let mut headers = HeaderMap::new();

            // 转发 Content-Type 头
            if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
                if let Ok(ct_str) = content_type.to_str() {
                    if let Ok(header_value) = ct_str.parse() {
                        headers.insert(axum::http::header::CONTENT_TYPE, header_value);
                    }
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

    // 获取当前可执行文件的路径
    let exe_path = std::env::current_exe()?;
    let current_dir: PathBuf = exe_path
        .parent()
        .map(|x| x.to_owned())
        .unwrap_or_else(|| PathBuf::from_str(".").unwrap());

    // 构建日志文件路径
    let log_file_name = format!("{}.log", exe_path.file_stem().unwrap().to_string_lossy());
    let log_file_path = current_dir.join(log_file_name);

    // 初始化日志系统
    // 设置默认日志级别为 info
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    // 使用 env_logger 将日志输出到文件
    let file = std::fs::File::create(&log_file_path).expect("Failed to create log file");
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Pipe(Box::new(file)))
        .format(|buf, record| {
            // 自定义日志格式
            use chrono::Local;
            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "{} | {} | {} | {}",
                now,
                record.module_path().unwrap_or("unknown"),
                record.level(),
                record.args()
            )
        })
        .init();

    info!("Starting Forward URL Proxy on http://localhost:{}", port);
    info!(
        "Example usage: http://localhost:{}?url=https%3A%2F%2Fgoogle.com&proxy=http%3A%2F%2Flocalhost%3A7890",
        port
    );

    // 构建应用程序状态 (AppState)
    let app_state = AppState {
        // 在这里创建 reqwest Client，可以配置一些默认值
        http_client: Client::new(),
    };

    // 构建 Axum 路由
    let app = Router::new()
        .route("/", get(forward_request))
        .with_state(app_state); // 将 app_state 传递给所有处理程序

    // 绑定地址并启动服务器
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    info!("listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) // 转换为 std::io::Error
}
