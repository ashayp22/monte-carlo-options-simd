use rand_distr::StandardNormal;
use rand::Rng;

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
    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();
    let add: f32 = nudt + sidt;
    let mut total_price: f32 = 0.0;

    for _ in 0..num_trials as i32 {
        let mut stock_price_mult = 1.0;

        for _ in 0..steps as i32 {
            let rand_val = rand::thread_rng().sample::<f32, _>(StandardNormal);
            stock_price_mult *= (add * rand_val).exp();
        }

        let price = spot * stock_price_mult - strike;

        if price > 0.0 {
            total_price += price;
        }
    }

    let call_option_price: f32 =
        (total_price / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

#[test]
fn valid_price() {
    let price = call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mc {}", price);
    assert_eq!(3.0 <= price && price <= 4.5, true);
}