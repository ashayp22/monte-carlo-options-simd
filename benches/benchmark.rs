use mcoptions::mc32x8;
use mcoptions::mc;
use mcoptions::mcfast;
use wide::*;

use criterion::{criterion_group, criterion_main, Criterion};

const START_SPOT : i32 = 50;
const END_SPOT : i32 = 162;
const NUM_TRIALS : f32 = 1000.0;
const NUM_STEPS : f32 = 100.0;
const STRIKE : f32 = 110.0;
const YEARS_TO_EXPIRY : f32 = 0.5;
const RISK_FREE_RATE : f32 = 0.05;
const DIVIDEND_YIELD : f32 = 0.02;
const VOLATILITY : f32 = 0.25;

fn criterion_benchmark1(c: &mut Criterion) {
    let spot_increment : f32x8 =  f32x8::from([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    let strike_f32x8 : f32x8 = f32x8::splat(STRIKE);
    let years_to_expiry_f32x8 : f32x8 = f32x8::splat(YEARS_TO_EXPIRY);
    let risk_free_rate_f32x8 : f32x8 = f32x8::splat(RISK_FREE_RATE);
    let dividend_yield_f32x8 : f32x8 = f32x8::splat(DIVIDEND_YIELD);
    let volatility_f32x8 : f32x8 = f32x8::splat(VOLATILITY);

    c.bench_function("monte carlo", |b| b.iter(|| {
        for spot in START_SPOT..END_SPOT {
            _ = mc::call_price(spot as f32, STRIKE, VOLATILITY, RISK_FREE_RATE, YEARS_TO_EXPIRY, DIVIDEND_YIELD, NUM_STEPS, NUM_TRIALS);
        }
    }));

    c.bench_function("monte carlo fast", |b| b.iter(|| {
        for spot in START_SPOT..END_SPOT {
            _ = mcfast::call_price(spot as f32, STRIKE, VOLATILITY, RISK_FREE_RATE, YEARS_TO_EXPIRY, DIVIDEND_YIELD, NUM_STEPS, NUM_TRIALS);
        }
    }));

    c.bench_function("monte carlo 32x8", |b| b.iter(|| {
        for i in (START_SPOT..END_SPOT).step_by(8) {
            let spot : f32x8 = f32x8::splat(i as f32) + spot_increment;
            _ = mc32x8::call_price(spot, strike_f32x8, volatility_f32x8, risk_free_rate_f32x8, years_to_expiry_f32x8, dividend_yield_f32x8, NUM_STEPS, NUM_TRIALS);
        }
    }));
}

criterion_group!(benches, criterion_benchmark1);
criterion_main!(benches);