use rand_distr::StandardNormal;
use rand::Rng;
use wide::*;
use simd_rand::portable::*;

fn get_rand_norm() -> f32 {
    rand::thread_rng().sample::<f32, _>(StandardNormal)
}

pub fn call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    rng : Xoshiro256PlusPlusX8
) -> f32 {
    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();
    let add: f32x8 = f32x8::splat(nudt + sidt);
    let strike_decrease = f32x8::splat(-strike);
    let zeros: f32x8 = f32x8::splat(0.0);

    let mut total_prices: f32x8 = f32x8::splat(0.0);

    for _ in 0..(num_trials as i32) / 8 {
        let mut stock_price_mult: f32x8 = f32x8::splat(spot);

        for _ in 0..steps as i32 {
            // TODO: generate this with SIMD?
            // let vals: [f32; 8] = [
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            //     get_rand_norm(),
            // ];
            // let random_mult: f32x8 = f32x8::from(vals);
            let random_mult: f32x8 = rng.next_f64x8() as f32x8;
            stock_price_mult = stock_price_mult * (add * random_mult).exp();
        }

        let price: f32x8 = stock_price_mult + strike_decrease;
        total_prices = total_prices + f32x8::fast_max(price, zeros);
    }

    // TODO: can we speed this sum?
    // let arr: [f32; 8] = total_prices.to_array();
    let total_price: f32 = total_prices.to_array().iter().sum();
    // let total_price: f32 = ((arr[0] + arr[1]) + (arr[2] + arr[3])) + ((arr[4] + arr[5]) + (arr[6] + arr[7]));

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
