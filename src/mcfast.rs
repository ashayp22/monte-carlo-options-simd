use wide::*;
use simd_rand::portable::*;
use crate::rand32x8::get_rand_uniform_f32x8;
use rand_core::{ RngCore, SeedableRng };
use crate::bs::black_scholes_call_price;

// TODO: 
// Antithetic variate method
// Covariate variate method
// Lookback option

pub fn call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    rng: &mut Xoshiro256PlusPlusX8
) -> f32 {
    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();
    let nusidt: f32 = nudt + sidt;

    let strike_f32x8 = f32x8::splat(strike);
    let spot_f32x8 = f32x8::splat(spot);
    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let nusidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2 * nusidt);

    let half_steps: i32 = (steps as i32) / 2;

    let mut total_prices: f32x8 = f32x8::splat(0.0);

    for _ in 0..(num_trials as i32) / 8 {
        let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

        for _ in 0..half_steps {
            let first_rand: f32x8 = get_rand_uniform_f32x8(rng);
            let second_rand: f32x8 = get_rand_uniform_f32x8(rng);
            let (sin_rand, cos_rand) = f32x8::sin_cos(two_pi * second_rand);

            stock_price_mult = f32x8::mul_add(
                (-first_rand.ln()).sqrt(),
                sin_rand + cos_rand,
                stock_price_mult
            );
        }

        total_prices += f32x8::fast_max(
            f32x8::mul_sub(spot_f32x8, (stock_price_mult * nusidt_two_sqrt).exp(), strike_f32x8),
            zeros
        );
    }

    let call_option_price: f32 =
        (total_prices.reduce_add() / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

pub fn put_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    rng: &mut Xoshiro256PlusPlusX8
) -> f32 {
    call_price(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        rng
    ) +
        strike -
        spot
}

#[test]
fn valid_price1() {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);
    let price = call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0, &mut rng);
    println!("mcfast 1 {} vs {}", price, actual_price);
    assert_eq!(actual_price - 1.25 <= price && price <= actual_price + 1.25, true);
}

#[test]
fn valid_price2() {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(90.0, 110.0, 0.2, 0.05, 1.0, 0.02);
    let price = call_price(90.0, 110.0, 0.2, 0.05, 1.0, 0.02, 100.0, 10000.0, &mut rng);
    println!("mcfast 2 {} vs {}", price, actual_price);
    assert_eq!(actual_price - 1.25 <= price && price <= actual_price + 1.25, true);
}

#[test]
fn valid_price3() {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(112.0, 110.0, 0.2, 0.05, 1.0, 0.02);
    let price = call_price(112.0, 110.0, 0.2, 0.05, 1.0, 0.02, 100.0, 1000.0, &mut rng);
    println!("mcfast 3 {} vs {}", price, actual_price);
    assert_eq!(actual_price - 1.25 <= price && price <= actual_price + 1.25, true);
}
