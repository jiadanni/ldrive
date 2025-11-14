/// Bandwidth rate limiter
///
/// Implements token bucket algorithm for rate limiting uploads and downloads

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    bytes_per_second: u64,
}

struct RateLimiterState {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `bytes_per_second` - Maximum bytes per second (0 = unlimited)
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: bytes_per_second as f64,
                last_update: Instant::now(),
            })),
            bytes_per_second,
        }
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.bytes_per_second > 0
    }

    /// Wait until enough tokens are available for the given number of bytes
    ///
    /// Returns immediately if rate limiting is disabled (bytes_per_second = 0)
    pub async fn acquire(&self, bytes: usize) {
        if self.bytes_per_second == 0 {
            // No rate limiting
            return;
        }

        loop {
            let mut state = self.state.lock().await;

            // Refill tokens based on time elapsed
            let now = Instant::now();
            let elapsed = now.duration_since(state.last_update).as_secs_f64();
            let new_tokens = elapsed * self.bytes_per_second as f64;
            state.tokens = (state.tokens + new_tokens).min(self.bytes_per_second as f64);
            state.last_update = now;

            // Check if we have enough tokens
            if state.tokens >= bytes as f64 {
                state.tokens -= bytes as f64;
                break;
            }

            // Calculate how long to wait for tokens
            let tokens_needed = bytes as f64 - state.tokens;
            let wait_time = tokens_needed / self.bytes_per_second as f64;
            let wait_duration = Duration::from_secs_f64(wait_time);

            drop(state); // Release lock before sleeping
            tokio::time::sleep(wait_duration).await;
        }
    }

    /// Update the rate limit
    pub async fn set_rate(&self, bytes_per_second: u64) {
        let mut state = self.state.lock().await;
        state.tokens = bytes_per_second as f64;
    }
}

/// Wrapper for rate-limited I/O operations
pub struct RateLimitedReader<R> {
    inner: R,
    limiter: Arc<RateLimiter>,
}

impl<R> RateLimitedReader<R> {
    pub fn new(inner: R, limiter: Arc<RateLimiter>) -> Self {
        Self { inner, limiter }
    }
}

impl<R: std::io::Read> std::io::Read for RateLimitedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        if bytes_read > 0 && self.limiter.is_enabled() {
            // Block on async in sync context (not ideal, but works for file I/O)
            tokio::runtime::Handle::current().block_on(async {
                self.limiter.acquire(bytes_read).await;
            });
        }
        Ok(bytes_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_rate_limiter_unlimited() {
        let limiter = RateLimiter::new(0);
        assert!(!limiter.is_enabled());

        let start = Instant::now();
        limiter.acquire(1000000).await;
        let elapsed = start.elapsed();

        // Should complete almost instantly
        assert!(elapsed < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        // 1000 bytes per second
        let limiter = RateLimiter::new(1000);
        assert!(limiter.is_enabled());

        let start = Instant::now();

        // Request 1000 bytes
        limiter.acquire(1000).await;
        let first_elapsed = start.elapsed();

        // First request should be immediate
        assert!(first_elapsed < Duration::from_millis(100));

        // Request another 1000 bytes
        limiter.acquire(1000).await;
        let second_elapsed = start.elapsed();

        // Second request should wait ~1 second
        assert!(second_elapsed >= Duration::from_millis(900));
        assert!(second_elapsed < Duration::from_millis(1200));
    }

    #[tokio::test]
    async fn test_rate_limiter_small_chunks() {
        // 1000 bytes per second
        let limiter = RateLimiter::new(1000);

        let start = Instant::now();

        // Request 10x 100 bytes
        for _ in 0..10 {
            limiter.acquire(100).await;
        }

        let elapsed = start.elapsed();

        // Should take about 1 second total
        assert!(elapsed >= Duration::from_millis(900));
        assert!(elapsed < Duration::from_millis(1200));
    }

    #[tokio::test]
    async fn test_rate_limiter_update() {
        let limiter = RateLimiter::new(1000);

        // Update to higher rate
        limiter.set_rate(2000).await;

        let start = Instant::now();
        limiter.acquire(2000).await;
        let elapsed = start.elapsed();

        // Should be immediate with new rate
        assert!(elapsed < Duration::from_millis(100));
    }
}
