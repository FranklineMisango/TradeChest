mod hjb;
mod market_data;
mod order_engine;
mod fpga_bridge;

// ...existing code...
use market_data::MarketDataFeed;
use order_engine::OrderEngine;
use fpga_bridge::FPGAEngine;
use std::ffi::CStr;
use std::os::raw::c_char;

pub struct TradingCore {
    // ...existing code...
    fpga_engine: FPGAEngine,
    market_feed: MarketDataFeed,
    order_engine: OrderEngine,
    usd_balance: std::sync::atomic::AtomicU64,
    btc_balance: std::sync::atomic::AtomicU64,
    initial_usd: f64,
    initial_btc: f64,
}

#[repr(C)]
pub struct Quote {
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub inventory: i32,
    pub market_bid: f64,
    pub market_ask: f64,
    pub spread: f64,
    pub usd_balance: f64,
    pub btc_balance: f64,
    pub pnl: f64,
    pub latency_us: u64,
}

#[no_mangle]
pub extern "C" fn create_core(symbol: *const c_char) -> *mut TradingCore {
    let symbol = unsafe { CStr::from_ptr(symbol).to_str().unwrap() };
    let core = TradingCore::new(symbol.to_string());
    Box::into_raw(Box::new(core))
}

#[no_mangle]
pub extern "C" fn start_market_data(core: *mut TradingCore) {
    unsafe { (*core).start_feed() };
}

#[no_mangle]
pub extern "C" fn get_current_quote(core: *const TradingCore) -> Quote {
    let start = std::time::Instant::now();
    let quote = unsafe { (*core).get_quote() };
    let latency = start.elapsed().as_micros() as u64;
    Quote { latency_us: latency, ..quote }
}

#[no_mangle]
pub extern "C" fn set_initial_portfolio(core: *mut TradingCore, usd: f64, btc: f64) {
    unsafe { (*core).set_portfolio(usd, btc) };
}

#[no_mangle]
pub extern "C" fn simulate_buy_trade(core: *mut TradingCore, quantity: i32) -> i32 {
    unsafe { 
        let price = (*core).market_feed.current_ask();
        if (*core).order_engine.execute_buy(quantity, price, &(*core).usd_balance, &(*core).btc_balance) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn simulate_sell_trade(core: *mut TradingCore, quantity: i32) -> i32 {
    unsafe { 
        let price = (*core).market_feed.current_bid();
        if (*core).order_engine.execute_sell(quantity, price, &(*core).usd_balance, &(*core).btc_balance) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn auto_trade(core: *mut TradingCore, result: *mut u8, len: i32) -> i32 {
    unsafe {
        let inventory = (*core).order_engine.inventory();
        let mid_price = (*core).market_feed.current_price();
        let volatility = (*core).market_feed.realized_volatility();
        
        // Professional dynamic inventory management
        let time_factor = (*core).time_to_close_factor(); // 1.0 at open, 0.0 at close
        let vol_scalar = (volatility / 0.3).clamp(0.5, 2.0); // Scale vs base 30% vol
        let _liquidity_factor = (*core).market_feed.liquidity_factor();
        
        // Dynamic target: reduce inventory as market close approaches
        let base_target = (*core).initial_btc as i32;
        let target_inventory = (base_target as f64 * time_factor * 0.8) as i32; // Max 80% of base
        
        // Dynamic threshold: higher in volatile markets, lower near close
        let base_threshold = 5.0;
        let rebalance_threshold = (base_threshold * vol_scalar * time_factor.max(0.2)) as i32;
        
        let deviation = inventory - target_inventory;
        let abs_deviation = deviation.abs();
        
        // Risk-based position sizing
        let max_trade_size = (abs_deviation as f64 * 0.3).ceil() as i32; // Trade 30% of excess
        let trade_size = max_trade_size.clamp(1, 5); // Min 1, Max 5 BTC per trade
        
        if abs_deviation > rebalance_threshold {
            let (action, success) = if deviation > 0 {
                // Over-inventory: sell
                let executed = (*core).order_engine.execute_sell(trade_size, mid_price, &(*core).usd_balance, &(*core).btc_balance);
                (format!("SELL {} BTC (inv:{}->{}, tgt:{}, thr:{})", trade_size, inventory, inventory-trade_size, target_inventory, rebalance_threshold), executed)
            } else {
                // Under-inventory: buy
                let executed = (*core).order_engine.execute_buy(trade_size, mid_price, &(*core).usd_balance, &(*core).btc_balance);
                (format!("BUY {} BTC (inv:{}->{}, tgt:{}, thr:{})", trade_size, inventory, inventory+trade_size, target_inventory, rebalance_threshold), executed)
            };
            
            let trade_msg = if success { action } else { "TRADE failed (insufficient balance)".to_string() };
            let msg_bytes = trade_msg.as_bytes();
            let copy_len = std::cmp::min(msg_bytes.len(), len as usize - 1);
            std::ptr::copy_nonoverlapping(msg_bytes.as_ptr(), result, copy_len);
            *result.add(copy_len) = 0;
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn destroy_core(core: *mut TradingCore) {
    unsafe { drop(Box::from_raw(core)) };
}

impl TradingCore {
    fn new(symbol: String) -> Self {
        Self {
            // ...existing code...
            fpga_engine: FPGAEngine::new(),
            market_feed: MarketDataFeed::new(symbol),
            order_engine: OrderEngine::new(),
            usd_balance: std::sync::atomic::AtomicU64::new(0),
            btc_balance: std::sync::atomic::AtomicU64::new(0),
            initial_usd: 0.0,
            initial_btc: 0.0,
        }
    }

    fn set_portfolio(&mut self, usd: f64, btc: f64) {
        self.initial_usd = usd;
        self.initial_btc = btc;
        self.usd_balance.store(usd.to_bits(), std::sync::atomic::Ordering::Relaxed);
        self.btc_balance.store(btc.to_bits(), std::sync::atomic::Ordering::Relaxed);
        self.order_engine.set_inventory(btc as i32);
    }

    fn start_feed(&mut self) {
        self.market_feed.start();
    }

    fn time_to_close_factor(&self) -> f64 {
        // Simulate trading day: 24h cycle, reduce inventory toward close
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let seconds_in_day = 86400;
        let day_progress = (now % seconds_in_day) as f64 / seconds_in_day as f64;
        
        // Higher factor early in day, lower near close
        if day_progress < 0.75 { 1.0 } else { (1.0 - day_progress) * 4.0 }
    }

    fn get_quote(&self) -> Quote {
        let mid_price = self.market_feed.current_price();
        let market_bid = self.market_feed.current_bid();
        let market_ask = self.market_feed.current_ask();
        let inventory = self.order_engine.inventory();
        // Use FPGA for optimal quote calculation
        let volatility = self.market_feed.realized_volatility();
        let (optimal_bid, optimal_ask, fpga_latency_ns) = self.fpga_engine.calculate_optimal_quotes(mid_price, inventory, volatility);
        
        let current_usd = f64::from_bits(self.usd_balance.load(std::sync::atomic::Ordering::Relaxed));
        let current_btc = f64::from_bits(self.btc_balance.load(std::sync::atomic::Ordering::Relaxed));
        // Realized P&L only (from actual trades, not mark-to-market)
        let realized_pnl = (current_usd - self.initial_usd) + (current_btc - self.initial_btc) * mid_price;
        
        Quote {
            bid: optimal_bid,
            ask: optimal_ask,
            mid: mid_price,
            inventory,
            market_bid,
            market_ask,
            spread: market_ask - market_bid,
            usd_balance: current_usd,
            btc_balance: current_btc,
            pnl: realized_pnl,
            latency_us: (fpga_latency_ns / 1000) as u64, // Convert ns to μs
        }
    }
}