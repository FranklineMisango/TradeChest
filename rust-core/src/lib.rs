mod hjb;
mod market_data;
mod order_engine;

use hjb::HJBEngine;
use market_data::MarketDataFeed;
use order_engine::OrderEngine;
use std::ffi::CStr;
use std::os::raw::c_char;

pub struct TradingCore {
    hjb_engine: HJBEngine,
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
pub extern "C" fn destroy_core(core: *mut TradingCore) {
    unsafe { drop(Box::from_raw(core)) };
}

impl TradingCore {
    fn new(symbol: String) -> Self {
        Self {
            hjb_engine: HJBEngine::new_default(),
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

    fn get_quote(&self) -> Quote {
        let mid_price = self.market_feed.current_price();
        let market_bid = self.market_feed.current_bid();
        let market_ask = self.market_feed.current_ask();
        let inventory = self.order_engine.inventory();
        let (optimal_bid, optimal_ask) = self.hjb_engine.optimal_quotes(0.0, mid_price, inventory);
        
        let current_usd = f64::from_bits(self.usd_balance.load(std::sync::atomic::Ordering::Relaxed));
        let current_btc = f64::from_bits(self.btc_balance.load(std::sync::atomic::Ordering::Relaxed));
        let portfolio_value = current_usd + current_btc * mid_price;
        let initial_value = self.initial_usd + self.initial_btc * mid_price;
        let pnl = portfolio_value - initial_value;
        
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
            pnl,
            latency_us: 0, // Set by caller
        }
    }
}