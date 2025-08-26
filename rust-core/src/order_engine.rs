use std::sync::atomic::{AtomicI32, Ordering};

pub struct OrderEngine {
    inventory: AtomicI32,
}

impl OrderEngine {
    pub fn new() -> Self {
        Self {
            inventory: AtomicI32::new(0),
        }
    }

    pub fn inventory(&self) -> i32 {
        self.inventory.load(Ordering::Relaxed)
    }

    pub fn execute_buy(&self, quantity: i32, price: f64, usd_balance: &std::sync::atomic::AtomicU64, btc_balance: &std::sync::atomic::AtomicU64) -> bool {
        let cost = quantity as f64 * price;
        let current_usd = f64::from_bits(usd_balance.load(Ordering::Relaxed));
        
        if current_usd >= cost {
            self.inventory.fetch_add(quantity, Ordering::Relaxed);
            usd_balance.store((current_usd - cost).to_bits(), Ordering::Relaxed);
            let current_btc = f64::from_bits(btc_balance.load(Ordering::Relaxed));
            btc_balance.store((current_btc + quantity as f64).to_bits(), Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn execute_sell(&self, quantity: i32, price: f64, usd_balance: &std::sync::atomic::AtomicU64, btc_balance: &std::sync::atomic::AtomicU64) -> bool {
        let current_btc = f64::from_bits(btc_balance.load(Ordering::Relaxed));
        
        if current_btc >= quantity as f64 {
            self.inventory.fetch_sub(quantity, Ordering::Relaxed);
            let proceeds = quantity as f64 * price;
            let current_usd = f64::from_bits(usd_balance.load(Ordering::Relaxed));
            usd_balance.store((current_usd + proceeds).to_bits(), Ordering::Relaxed);
            btc_balance.store((current_btc - quantity as f64).to_bits(), Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn set_inventory(&self, value: i32) {
        self.inventory.store(value, Ordering::Relaxed);
    }
}