use rand_distr::StandardNormal;
use rand::Rng;
use wide::*;
use simd_rand::portable::*;
use rand_core::{ RngCore, SeedableRng };

fn get_rand_uniform_f32x8(rng: &mut Xoshiro256PlusPlusX8) -> f32x8 {
    let rand_f64x8: [f64; 8] = rng.next_f64x8().to_array();
    let random_mult = f32x8::from([
        rand_f64x8[0] as f32,
        rand_f64x8[1] as f32,
        rand_f64x8[2] as f32,
        rand_f64x8[3] as f32,
        rand_f64x8[4] as f32,
        rand_f64x8[5] as f32,
        rand_f64x8[6] as f32,
        rand_f64x8[7] as f32,
    ]);

    random_mult
}

pub fn call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();
    let add: f32x8 = f32x8::splat(nudt + sidt);
    let strike_decrease = f32x8::splat(-strike);
    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let neg_two = f32x8::splat(-2.0);

    let mut total_prices: f32x8 = f32x8::splat(0.0);

    for _ in 0..(num_trials as i32) / 8 {
        let mut stock_price_mult: f32x8 = f32x8::splat(spot);

        for _ in 0..(steps as i32) / 2 {
            let first: f32x8 = get_rand_uniform_f32x8(&mut rng);
            let second: f32x8 = get_rand_uniform_f32x8(&mut rng);    

            let first_random_mult = (neg_two * first.ln()).sqrt() * (two_pi * second).cos();
            let second_random_mult = (neg_two * first.ln()).sqrt() * (two_pi * second).sin();

            stock_price_mult = stock_price_mult * (add * first_random_mult).exp() * (add * second_random_mult).exp();
        }

        let price: f32x8 = stock_price_mult + strike_decrease;
        total_prices = total_prices + f32x8::fast_max(price, zeros);
    }

    let total_price: f32 = total_prices.reduce_add();

    let call_option_price: f32 =
        (total_price / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

#[test]
fn valid_price() {
    let price = call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mcfast {}", price);
    assert_eq!(3.5 <= price && price <= 4.5, true);
}
