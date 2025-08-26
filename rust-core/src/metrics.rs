use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct LatencyMetrics {
    quote_latency_sum: AtomicU64,
    quote_count: AtomicU64,
    max_quote_latency: AtomicU64,
    order_latency_sum: AtomicU64,
    order_count: AtomicU64,
    max_order_latency: AtomicU64,
}

impl LatencyMetrics {
    pub fn new() -> Self {
        Self {
            quote_latency_sum: AtomicU64::new(0),
            quote_count: AtomicU64::new(0),
            max_quote_latency: AtomicU64::new(0),
            order_latency_sum: AtomicU64::new(0),
            order_count: AtomicU64::new(0),
            max_order_latency: AtomicU64::new(0),
        }
    }

    pub fn record_quote_latency(&self, latency: Duration) {
        let micros = latency.as_micros() as u64;
        self.quote_latency_sum.fetch_add(micros, Ordering::Relaxed);
        self.quote_count.fetch_add(1, Ordering::Relaxed);
        
        let mut current_max = self.max_quote_latency.load(Ordering::Relaxed);
        while micros > current_max {
            match self.max_quote_latency.compare_exchange_weak(
                current_max, micros, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    pub fn record_order_latency(&self, latency: Duration) {
        let micros = latency.as_micros() as u64;
        self.order_latency_sum.fetch_add(micros, Ordering::Relaxed);
        self.order_count.fetch_add(1, Ordering::Relaxed);
        
        let mut current_max = self.max_order_latency.load(Ordering::Relaxed);
        while micros > current_max {
            match self.max_order_latency.compare_exchange_weak(
                current_max, micros, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    pub fn avg_quote_latency_us(&self) -> f64 {
        let sum = self.quote_latency_sum.load(Ordering::Relaxed);
        let count = self.quote_count.load(Ordering::Relaxed);
        if count > 0 { sum as f64 / count as f64 } else { 0.0 }
    }

    pub fn max_quote_latency_us(&self) -> u64 {
        self.max_quote_latency.load(Ordering::Relaxed)
    }

    pub fn avg_order_latency_us(&self) -> f64 {
        let sum = self.order_latency_sum.load(Ordering::Relaxed);
        let count = self.order_count.load(Ordering::Relaxed);
        if count > 0 { sum as f64 / count as f64 } else { 0.0 }
    }

    pub fn max_order_latency_us(&self) -> u64 {
        self.max_order_latency.load(Ordering::Relaxed)
    }
}

pub struct LatencyTimer {
    start: Instant,
}

impl LatencyTimer {
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}