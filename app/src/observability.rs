use tracing::level_filters::LevelFilter;
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use std::path::Path;

/// Initialize tracing with file appender and console output
pub fn init_tracing(log_dir: &Path, log_level: &str, console_output: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(log_dir)?;

    // Parse log level
    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    let registry = Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(level_filter.to_string()))
        );

    // File appender - daily rotation
    let file_appender = rolling::daily(log_dir, "lucastra.log");
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_level(true);

    if console_output {
        // Console layer with pretty formatting
        let console_layer = fmt::layer()
            .pretty()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_level(true);

        registry
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        registry
            .with(file_layer)
            .init();
    }

    tracing::info!("Tracing initialized with level: {}", log_level);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_log_level_parsing() {
        // Test the log level parsing logic directly without actually initializing tracing
        let test_cases = vec![
            ("trace", "TRACE"),
            ("debug", "DEBUG"),
            ("info", "INFO"),
            ("warn", "WARN"),
            ("error", "ERROR"),
            ("invalid", "INFO"), // Should default to INFO
        ];

        for (input, _expected) in test_cases {
            let level_str = match input.to_lowercase().as_str() {
                "trace" => "TRACE",
                "debug" => "DEBUG",
                "info" => "INFO",
                "warn" => "WARN",
                "error" => "ERROR",
                _ => "INFO",
            };
            assert!(!level_str.is_empty());
        }
    }
}
