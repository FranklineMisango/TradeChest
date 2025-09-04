use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::ffi::CString;

extern "C" {
    fn create_core(symbol: *const std::os::raw::c_char) -> *mut std::ffi::c_void;
    fn get_current_quote(core: *const std::ffi::c_void) -> [f64; 12];
    fn set_initial_portfolio(core: *mut std::ffi::c_void, usd: f64, btc: f64);
    fn simulate_buy_trade(core: *mut std::ffi::c_void, quantity: i32) -> i32;
    fn simulate_sell_trade(core: *mut std::ffi::c_void, quantity: i32) -> i32;
    fn destroy_core(core: *mut std::ffi::c_void);
}

fn benchmark_quote_generation(c: &mut Criterion) {
    let symbol = CString::new("BTCUSDT").unwrap();
    let core = unsafe { create_core(symbol.as_ptr()) };
    
    c.bench_function("quote_generation", |b| {
        b.iter(|| {
            black_box(unsafe { get_current_quote(core) })
        })
    });
    
    unsafe { destroy_core(core) };
}

fn benchmark_end_to_end_latency(c: &mut Criterion) {
    let symbol = CString::new("BTCUSDT").unwrap();
    let core = unsafe { create_core(symbol.as_ptr()) };
    unsafe { set_initial_portfolio(core, 10000.0, 1.0) };
    
    c.bench_function("end_to_end_trade", |b| {
        b.iter(|| {
            black_box(unsafe { 
                simulate_buy_trade(core, 1);
                get_current_quote(core);
            })
        })
    });
    
    unsafe { destroy_core(core) };
}

fn benchmark_order_execution(c: &mut Criterion) {
    let symbol = CString::new("BTCUSDT").unwrap();
    let core = unsafe { create_core(symbol.as_ptr()) };
    unsafe { set_initial_portfolio(core, 10000.0, 1.0) };
    
    c.bench_function("buy_order", |b| {
        b.iter(|| {
            black_box(unsafe { simulate_buy_trade(core, 1) })
        })
    });
    
    c.bench_function("sell_order", |b| {
        b.iter(|| {
            black_box(unsafe { simulate_sell_trade(core, 1) })
        })
    });
    
    unsafe { destroy_core(core) };
}

criterion_group!(benches, benchmark_quote_generation, benchmark_order_execution, benchmark_end_to_end_latency);
criterion_main!(benches);