use std::sync::{Arc, RwLock};
use std::thread;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use serde_json::Value;
use url::Url;

pub struct MarketDataFeed {
    _symbol: String,
    price: Arc<RwLock<f64>>,
    bid: Arc<RwLock<f64>>,
    ask: Arc<RwLock<f64>>,
    price_history: Arc<RwLock<Vec<f64>>>,
}

impl MarketDataFeed {
    pub fn new(symbol: String) -> Self {
        Self {
            _symbol: symbol,
            price: Arc::new(RwLock::new(0.0)),
            bid: Arc::new(RwLock::new(0.0)),
            ask: Arc::new(RwLock::new(0.0)),
            price_history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }

    pub fn start(&mut self) {
        let symbol_lower = self._symbol.to_lowercase();
        let price = self.price.clone();
        let bid = self.bid.clone();
        let ask = self.ask.clone();
        let price_history = self.price_history.clone();
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let ticker_url = format!("wss://stream.binance.com:9443/ws/{}@ticker", symbol_lower);
                let depth_url = format!("wss://stream.binance.com:9443/ws/{}@depth5@100ms", symbol_lower);
                
                // Start ticker stream
                let price_clone = price.clone();
                let price_history_clone = price_history.clone();
                tokio::spawn(async move {
                    loop {
                        if let Ok(url) = Url::parse(&ticker_url) {
                            if let Ok((ws_stream, _)) = connect_async(url).await {
                                let (mut _write, mut read) = ws_stream.split();
                                while let Some(msg) = read.next().await {
                                    if let Ok(Message::Text(text)) = msg {
                                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                            if let Some(last_price) = data["c"].as_str() {
                                                if let Ok(price_val) = last_price.parse::<f64>() {
                                                    let mut price_lock = price_clone.write().unwrap();
                                                    let _old_price = *price_lock;
                                                    *price_lock = price_val;
                                                    drop(price_lock);
                                                    
                                                    // Update price history
                                                    let mut history = price_history_clone.write().unwrap();
                                                    history.push(price_val);
                                                    if history.len() > 1000 {
                                                        history.remove(0);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                });
                
                // Start depth stream
                loop {
                    if let Ok(url) = Url::parse(&depth_url) {
                        if let Ok((ws_stream, _)) = connect_async(url).await {
                            let (mut _write, mut read) = ws_stream.split();
                            while let Some(msg) = read.next().await {
                                if let Ok(Message::Text(text)) = msg {
                                    if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                        if let Some(bids) = data["bids"].as_array() {
                                            if let Some(asks) = data["asks"].as_array() {
                                                if let (Some(best_bid), Some(best_ask)) = (bids.first(), asks.first()) {
                                                    if let (Some(bid_price), Some(ask_price)) = 
                                                        (best_bid[0].as_str(), best_ask[0].as_str()) {
                                                        if let (Ok(bid_val), Ok(ask_val)) = 
                                                            (bid_price.parse::<f64>(), ask_price.parse::<f64>()) {
                                                            *bid.write().unwrap() = bid_val;
                                                            *ask.write().unwrap() = ask_val;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        });
    }

    pub fn current_price(&self) -> f64 {
        *self.price.read().unwrap()
    }
    
    pub fn current_bid(&self) -> f64 {
        *self.bid.read().unwrap()
    }
    
    pub fn current_ask(&self) -> f64 {
        *self.ask.read().unwrap()
    }
    
    pub fn realized_volatility(&self) -> f64 {
        let prices = self.price_history.read().unwrap();
        if prices.len() < 2 { return 0.3; } // Default 30% vol
        
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1] / w[0]).ln())
            .collect();
        
        if returns.is_empty() { return 0.3; }
        
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        (variance.sqrt() * (252.0_f64).sqrt()).clamp(0.1, 2.0) // Annualized
    }
    
    pub fn liquidity_factor(&self) -> f64 {
        let bid = self.current_bid();
        let ask = self.current_ask();
        if bid <= 0.0 || ask <= 0.0 { return 1.0; }
        
        let spread_bps = ((ask - bid) / ((ask + bid) / 2.0)) * 10000.0;
        (20.0 / spread_bps).clamp(0.5, 2.0) // Higher factor = more liquid
    }
}