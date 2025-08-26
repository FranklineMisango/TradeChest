// use ndarray::{Array1, Array3}; // Unused imports

pub struct HJBEngine {
    sigma: f64,
    gamma: f64,
    k: f64,
    _c: f64,
}

impl HJBEngine {
    pub fn new_default() -> Self {
        Self { sigma: 0.3, gamma: 0.1, k: 1.5, _c: 1.0 }
    }

    pub fn optimal_quotes(&self, t: f64, s: f64, inventory: i32) -> (f64, f64) {
        let remaining_time = 1.0 - t;
        let bid_spread = (2 * inventory + 1) as f64 * self.gamma * self.sigma.powi(2) * remaining_time / 2.0 
                        + (1.0 + self.gamma / self.k).ln() / self.gamma;
        let ask_spread = (1 - 2 * inventory) as f64 * self.gamma * self.sigma.powi(2) * remaining_time / 2.0 
                        + (1.0 + self.gamma / self.k).ln() / self.gamma;

        (s - bid_spread, s + ask_spread)
    }
}