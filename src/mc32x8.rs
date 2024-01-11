use rand_distr::StandardNormal;
use rand::Rng;
use wide::*;

fn get_rand_norm() -> f32 {
    rand::thread_rng().sample::<f32, _>(StandardNormal)
}

pub fn call_price(
    spot: f32x8,
    strike: f32x8,
    volatility: f32x8,
    risk_free_rate: f32x8,
    years_to_expiry: f32x8,
    dividend_yield: f32x8,
    steps: f32,
    num_trials: f32
) -> f32x8 {
    let dt: f32x8 = years_to_expiry / steps;
    let nudt: f32x8 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32x8 = volatility * dt.sqrt();
    let add: f32x8 = nudt + sidt;

    let zeros: f32x8 = f32x8::splat(0.0);
    let mut total_price: f32x8 = f32x8::splat(0.0);

    for _ in 0..num_trials as i32 {
        let mut stock_price_mult = f32x8::splat(1.0);

        for _ in 0..steps as i32 {
            let vals: [f32; 8] = [
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
                get_rand_norm(),
            ];
            let random_mult: f32x8 = f32x8::from(vals);

            stock_price_mult *= (add * random_mult).exp();
        }

        let price = spot * stock_price_mult - strike;
        total_price = total_price + f32x8::fast_max(price, zeros);
    }

    let call_option_price: f32x8 =
        (total_price / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

#[test]
fn valid_price() {
    use bytemuck::cast;

    let price: [f32; 8] = cast(
        call_price(
            f32x8::splat(100.0),
            f32x8::splat(110.0),
            f32x8::splat(0.25),
            f32x8::splat(0.05),
            f32x8::splat(0.5),
            f32x8::splat(0.02),
            100.0,
            1000.0
        )
    );
    println!("mc32x8 {}", price[0]);
    assert_eq!(3.5 <= price[0] && price[0] <= 4.5, true);
}
