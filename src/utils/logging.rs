use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing::Level;

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    // 设置日志过滤器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::default()
                .add_directive(Level::DEBUG.into())
                .add_directive("actix_web=info".parse().unwrap())
                .add_directive("sqlx=warn".parse().unwrap())
        });

    // 彩色控制台输出
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_timer(fmt::time::ChronoUtc::rfc_3339())
        .pretty();

    // JSON文件输出
    let file_appender = tracing_appender::rolling::daily("/var/log/notes_server", "notes.log");
    let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    file_guard
}

// 自定义日志格式宏
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr, $status:expr, $duration:expr, $ip:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            duration_ms = %$duration.as_millis(),
            ip = %$ip,
            "Request completed"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = %$context,
            "Operation failed"
        );
    };
}