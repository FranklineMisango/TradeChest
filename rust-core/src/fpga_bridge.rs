// ...existing code...

pub struct FPGAEngine {
    _initialized: bool,
}

impl FPGAEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            _initialized: false,
        };
        engine.initialize();
        engine
    }

    fn initialize(&mut self) {
        // For now, use software fallback (Verilator integration in progress)
        self._initialized = false;
    }

    pub fn calculate_optimal_quotes(&self, mid_price: f64, inventory: i32, volatility: f64, gamma: f64, k: f64) -> (f64, f64, u32) {
        if !self._initialized {
            return self.software_fallback(mid_price, inventory, volatility, 0.1, 1.5);
        }

    // ...existing code...
        
        // FPGA disabled - use optimized software calculation
        self.software_fallback(mid_price, inventory, volatility, gamma, k)
    }

    fn software_fallback(&self, mid_price: f64, inventory: i32, volatility: f64, gamma: f64, k: f64) -> (f64, f64, u32) {
        // Optimized software HJB implementation
    // ...existing code...
        
        let inv_f64 = inventory as f64;
        
        // Vectorized calculations
    let vol_sq = volatility * volatility;
    let gamma_vol_sq = gamma * vol_sq;
    let reservation_price = mid_price - inv_f64 * gamma_vol_sq;

    // Precomputed logarithm
    let ln_term = (1.0 + gamma / k).ln();
    let spread = gamma_vol_sq + (2.0 / gamma) * ln_term;

    let half_spread = spread * 0.5;
    let optimal_bid = reservation_price - half_spread;
    let optimal_ask = reservation_price + half_spread;

    // Latency measurement removed; return 0 as placeholder
    let latency_ns = 0u32;
    (optimal_bid, optimal_ask, latency_ns)
    }
}