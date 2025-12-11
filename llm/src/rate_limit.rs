//! Rate limiting for API calls to prevent hitting provider limits.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Simple token bucket rate limiter.
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    /// 
    /// # Arguments
    /// * `requests_per_minute` - Maximum requests per minute
    pub fn new(requests_per_minute: u32) -> Self {
        let max_tokens = requests_per_minute as f64;
        let refill_rate = requests_per_minute as f64 / 60.0; // tokens per second

        Self {
            tokens: Arc::new(Mutex::new(max_tokens)),
            max_tokens,
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Wait until a token is available, then consume it.
    pub async fn acquire(&self) {
        loop {
            {
                let mut tokens = self.tokens.lock().await;
                let mut last_refill = self.last_refill.lock().await;

                // Refill tokens based on elapsed time
                let now = Instant::now();
                let elapsed = now.duration_since(*last_refill).as_secs_f64();
                let new_tokens = elapsed * self.refill_rate;
                *tokens = (*tokens + new_tokens).min(self.max_tokens);
                *last_refill = now;

                // If token available, consume and return
                if *tokens >= 1.0 {
                    *tokens -= 1.0;
                    return;
                }
            }

            // Sleep a bit before retrying
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Try to acquire without waiting. Returns true if acquired, false if would block.
    pub async fn try_acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // Refill tokens
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        *tokens = (*tokens + new_tokens).min(self.max_tokens);
        *last_refill = now;

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(60); // 60 requests/min = 1 req/sec
        
        // Should acquire immediately (bucket starts full)
        limiter.acquire().await;
        assert!(limiter.try_acquire().await); // Should still have tokens
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(60); // 1 req/sec
        
        // Drain all tokens
        for _ in 0..60 {
            limiter.acquire().await;
        }

        // Should not have tokens immediately
        assert!(!limiter.try_acquire().await);

        // Wait for refill (1 second should give ~1 token)
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(limiter.try_acquire().await);
    }
}
