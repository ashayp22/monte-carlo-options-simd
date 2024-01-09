use rand_distr::StandardNormal;
use rand::Rng;

fn erf(x : f64) -> f64 {
    let t = x.signum();
    let e = x.abs();
    const N: f64 = 0.3275911;
    const A: f64 = 0.254829592;
    const R: f64 = -0.284496736;
    const I: f64 = 1.421413741;
    const L: f64 = -1.453152027;
    const D: f64 = 1.061405429;
    let u = 1.0 / (1.0 + N * e);
    let m = 1.0 - ((((D * u + L) * u + I) * u + R) * u + A) * u * (-e * e).exp();
    t * m
}

fn normal_cdf(x : f64) -> f64 {
    if x < -1.0e6 {
        0.0
    } else if x > 1.0e6 {
        1.0
    } else {
        0.5 * (1.0 + erf(x / 2.0f64.sqrt()))
    }
}

fn black_scholes_call_price(
    spot: f64,
    strike: f64,
    volatility: f64,
    risk_free_rate: f64,
    years_to_expiry: f64,
    dividend_yield: f64
) -> f64 {
    let d1 : f64 = ((spot / strike).ln() + (risk_free_rate - dividend_yield + ((volatility * volatility) / 2.0))* years_to_expiry) / (volatility * years_to_expiry.sqrt());
    let d2 : f64 = d1 - volatility * years_to_expiry.sqrt();

    let call : f64 = spot * (-dividend_yield * years_to_expiry).exp() * normal_cdf(d1) - strike * (-risk_free_rate * years_to_expiry).exp() * normal_cdf(d2);
    call
}

fn monte_carlo_call_price(
    spot: f64,
    strike : f64,
    volatility: f64,
    risk_free_rate: f64,
    years_to_expiry: f64,
    dividend_yield: f64,
    steps : f64,
    num_trials : f64
) -> f64 {
    let dt : f64 = years_to_expiry / steps;
    let nudt : f64 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt : f64 = volatility * dt.sqrt();
    let add : f64 = nudt + sidt;
    let mut total_price : f64 = 0.0;

    for _ in 0..(num_trials as i32) {
        let mut curr_stock_price = spot;
        for _ in 0..(steps as i32) {
            let rand_val = rand::thread_rng().sample::<f64,_>(StandardNormal);
            curr_stock_price *= (add * rand_val).exp();
        }

        let price = curr_stock_price - strike;

        if price > 0.0 {
            total_price += price;
        }
    }
    
    let call_option_price : f64 = total_price / num_trials * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

fn main() {
    let spot : f64 = 100.0;
    let strike : f64 = 110.0;
    let years_to_expiry = 0.5;
    let risk_free_rate : f64 = 0.05;
    let dividend_yield : f64 = 0.02;
    let volatility : f64 = 0.25;
    let num_trials : f64 = 1000.0;
    let steps : f64 = 100.0;

    println!("Black-scholes value: {}", black_scholes_call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield));
    println!("Monte carlo value: {}", monte_carlo_call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, steps, num_trials));
}
