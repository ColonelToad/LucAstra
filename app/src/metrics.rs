use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use serde::Serialize;

/// Metrics collector for system observability
#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    command_count: AtomicU64,
    tool_success_count: AtomicU64,
    tool_failure_count: AtomicU64,
    search_queries: AtomicU64,
    total_search_latency_ms: AtomicU64,
    app_startup_time_ms: AtomicU64,
    custom_counters: std::sync::Mutex<HashMap<String, u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub command_count: u64,
    pub tool_success_count: u64,
    pub tool_failure_count: u64,
    pub search_queries: u64,
    pub average_search_latency_ms: u64,
    pub app_startup_time_ms: u64,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Metrics {
            inner: Arc::new(MetricsInner {
                command_count: AtomicU64::new(0),
                tool_success_count: AtomicU64::new(0),
                tool_failure_count: AtomicU64::new(0),
                search_queries: AtomicU64::new(0),
                total_search_latency_ms: AtomicU64::new(0),
                app_startup_time_ms: AtomicU64::new(0),
                custom_counters: std::sync::Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Record a command execution
    pub fn record_command(&self) {
        self.inner.command_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful tool execution
    pub fn record_tool_success(&self) {
        self.inner.tool_success_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed tool execution
    pub fn record_tool_failure(&self) {
        self.inner.tool_failure_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a search query with latency
    pub fn record_search(&self, latency_ms: u64) {
        self.inner.search_queries.fetch_add(1, Ordering::Relaxed);
        self.inner.total_search_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    /// Record app startup time
    pub fn record_startup_time(&self, startup_ms: u64) {
        self.inner.app_startup_time_ms.store(startup_ms, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let command_count = self.inner.command_count.load(Ordering::Relaxed);
        let tool_success_count = self.inner.tool_success_count.load(Ordering::Relaxed);
        let tool_failure_count = self.inner.tool_failure_count.load(Ordering::Relaxed);
        let search_queries = self.inner.search_queries.load(Ordering::Relaxed);
        let total_search_latency_ms = self.inner.total_search_latency_ms.load(Ordering::Relaxed);
        let app_startup_time_ms = self.inner.app_startup_time_ms.load(Ordering::Relaxed);

        let average_search_latency_ms = if search_queries > 0 {
            total_search_latency_ms / search_queries
        } else {
            0
        };

        MetricsSnapshot {
            command_count,
            tool_success_count,
            tool_failure_count,
            search_queries,
            average_search_latency_ms,
            app_startup_time_ms,
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.inner.command_count.store(0, Ordering::Relaxed);
        self.inner.tool_success_count.store(0, Ordering::Relaxed);
        self.inner.tool_failure_count.store(0, Ordering::Relaxed);
        self.inner.search_queries.store(0, Ordering::Relaxed);
        self.inner.total_search_latency_ms.store(0, Ordering::Relaxed);
        self.inner.app_startup_time_ms.store(0, Ordering::Relaxed);
        let _ = self.inner.custom_counters.lock().map(|mut m| m.clear());
    }

    /// Get tool success rate as percentage (0-100)
    pub fn tool_success_rate(&self) -> f64 {
        let success = self.inner.tool_success_count.load(Ordering::Relaxed) as f64;
        let failure = self.inner.tool_failure_count.load(Ordering::Relaxed) as f64;
        let total = success + failure;

        if total == 0.0 {
            0.0
        } else {
            (success / total) * 100.0
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.command_count, 0);
        assert_eq!(snapshot.tool_success_count, 0);
    }

    #[test]
    fn test_record_command() {
        let metrics = Metrics::new();
        metrics.record_command();
        metrics.record_command();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.command_count, 2);
    }

    #[test]
    fn test_record_tool_success_failure() {
        let metrics = Metrics::new();
        metrics.record_tool_success();
        metrics.record_tool_success();
        metrics.record_tool_failure();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.tool_success_count, 2);
        assert_eq!(snapshot.tool_failure_count, 1);
    }

    #[test]
    fn test_tool_success_rate() {
        let metrics = Metrics::new();
        metrics.record_tool_success();
        metrics.record_tool_success();
        metrics.record_tool_success();
        metrics.record_tool_failure();
        assert!((metrics.tool_success_rate() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_search_latency() {
        let metrics = Metrics::new();
        metrics.record_search(100);
        metrics.record_search(200);
        metrics.record_search(300);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.search_queries, 3);
        assert_eq!(snapshot.average_search_latency_ms, 200);
    }

    #[test]
    fn test_reset_metrics() {
        let metrics = Metrics::new();
        metrics.record_command();
        metrics.record_tool_success();
        metrics.reset();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.command_count, 0);
        assert_eq!(snapshot.tool_success_count, 0);
    }
}
