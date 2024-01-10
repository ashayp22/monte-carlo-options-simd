use rand_distr::StandardNormal;
use rand::Rng;
use wide::*;

/*
TODO:


- Switch benchmarks
- vectorize erf, normal cdf, monte_carlo_price

- vectorize random number generation

QUESTIONS:

- how do we speed up the two for loops?
*/

fn erf(x : f32) -> f32 {
    let t = x.signum();
    let e = x.abs();
    const N: f32 = 0.3275911;
    const A: f32 = 0.254829592;
    const R: f32 = -0.284496736;
    const I: f32 = 1.421413741;
    const L: f32 = -1.453152027;
    const D: f32 = 1.061405429;
    let u = 1.0 / (1.0 + N * e);
    let m = 1.0 - ((((D * u + L) * u + I) * u + R) * u + A) * u * (-e * e).exp();
    t * m
}

fn normal_cdf(x : f32) -> f32 {
    0.5 * (1.0 + erf(x / 2.0f32.sqrt()))
}

fn black_scholes_call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let d1 : f32 = ((spot / strike).ln() + (risk_free_rate - dividend_yield + ((volatility * volatility) / 2.0))* years_to_expiry) / (volatility * years_to_expiry.sqrt());
    let d2 : f32 = d1 - volatility * years_to_expiry.sqrt();

    let call : f32 = spot * (-dividend_yield * years_to_expiry).exp() * normal_cdf(d1) - strike * (-risk_free_rate * years_to_expiry).exp() * normal_cdf(d2);
    call
}

fn monte_carlo_call_price(
    spot: f32,
    strike : f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps : f32,
    num_trials : f32
) -> f32 {
    let dt : f32 = years_to_expiry / steps;
    let nudt : f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt : f32 = volatility * dt.sqrt();
    let add : f32 = nudt + sidt;
    let mut total_price : f32 = 0.0;

    for _ in 0..(num_trials as i32) {
        let mut stock_price_mult = 1.0;

        for _ in 0..(steps as i32) {
            let rand_val = rand::thread_rng().sample::<f32,_>(StandardNormal);
            stock_price_mult *= (add * rand_val).exp();
        }

        let price = spot * stock_price_mult - strike;

        if price > 0.0 {
            total_price += price;
        }
    }
    
    let call_option_price : f32 = total_price / num_trials * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

fn monte_carlo_call_price_f32x8(
    spot: f32x8,
    strike : f32x8,
    volatility: f32x8,
    risk_free_rate: f32x8,
    years_to_expiry: f32x8,
    dividend_yield: f32x8,
    steps : f32,
    num_trials : f32
) -> f32x8 {
    let dt : f32x8 = years_to_expiry / steps;
    let nudt : f32x8 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt : f32x8 = volatility * dt.sqrt();
    let add : f32x8 = nudt + sidt;

    let zeros : f32x8 = f32x8::splat(0.0);
    let mut total_price : f32x8 = f32x8::splat(0.0);

    for _ in 0..(num_trials as i32) {
        let mut stock_price_mult = f32x8::splat(1.0);

        for _ in 0..(steps as i32) {
            let vals : [f32 ; 8] = [get_rand_norm(), get_rand_norm(),  get_rand_norm(), get_rand_norm(), get_rand_norm(), get_rand_norm(),  get_rand_norm(), get_rand_norm() ];
            let random_mult : f32x8 = f32x8::from(vals);

            stock_price_mult *= (add * random_mult).exp();
        }

        let price = spot * stock_price_mult - strike;
        total_price = total_price + f32x8::fast_max(price, zeros);
    }
    
    let call_option_price : f32x8 = total_price / num_trials * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

fn get_rand_norm() -> f32 {
    rand::thread_rng().sample::<f32,_>(StandardNormal)
}

fn fast_monte_carlo_call_price(
    spot: f32,
    strike : f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps : f32,
    num_trials : f32
) -> f32 {
    let dt : f32 = years_to_expiry / steps;
    let nudt : f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt : f32 = volatility * dt.sqrt();
    let add : f32x8 = f32x8::splat(nudt + sidt);
    let strike_decrease = f32x8::splat(-strike);
    let spot_mult = f32x8::splat(spot);

    let mut total_price : f32 = 0.0;

    // TODO: figure out how to make this faster

    for _ in 0..(num_trials as i32)/8 {
        let mut stock_price_mult : f32x8 = f32x8::splat(spot);

        for _ in 0..(steps as i32) {
            let vals : [f32 ; 8] = [get_rand_norm(), get_rand_norm(),  get_rand_norm(), get_rand_norm(), get_rand_norm(), get_rand_norm(),  get_rand_norm(), get_rand_norm() ];
            let random_mult : f32x8 = f32x8::from(vals);

            stock_price_mult *= random_mult;
            stock_price_mult *= add;
        }

        // TODO: update total_price
        let price : f32x8 = spot_mult * stock_price_mult + strike_decrease;

        // if price > 0.0 {
        //     total_price += price;
        // }
    }
    
    let call_option_price : f32 = total_price / num_trials * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

// #[test]
// fn check_black_scholes() {
//     let spot : f32 = 100.0;
//     let strike : f32 = 110.0;
//     let years_to_expiry = 0.5;
//     let risk_free_rate : f32 = 0.05;
//     let dividend_yield : f32 = 0.02;
//     let volatility : f32 = 0.25;

//     let now = std::time::Instant::now();

//     let price = black_scholes_call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);

//     let duration = now.elapsed().as_millis();
//     println!("Time take [Black-scholes] {}ms {}", duration, price);
// }

#[test]
fn check_monte_carlo() {
    let strike : f32 = 110.0;
    let years_to_expiry = 0.5;
    let risk_free_rate : f32 = 0.05;
    let dividend_yield : f32 = 0.02;
    let volatility : f32 = 0.25;
    let num_trials : f32 = 1000.0;
    let steps : f32 = 100.0;

    let now = std::time::Instant::now();

    for spot in 50..162 {
        _ = monte_carlo_call_price(spot as f32, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, steps, num_trials);
    }

    let duration = now.elapsed().as_millis();
    println!("Time take [Monte-carlo] {}ms", duration);
}

#[test]
fn check_fast_monte_carlo() {
    let strike : f32 = 110.0;
    let years_to_expiry = 0.5;
    let risk_free_rate : f32 = 0.05;
    let dividend_yield : f32 = 0.02;
    let volatility : f32 = 0.25;
    let num_trials : f32 = 1000.0;
    let steps : f32 = 100.0;

    let now = std::time::Instant::now();

    for spot in 50..162 {
        _ = fast_monte_carlo_call_price(spot as f32, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, steps, num_trials);
    }

    let duration = now.elapsed().as_millis();
    println!("Time take [Fast Monte-carlo] {}ms", duration);
}


#[test]
fn check_monte_carlo_f32x8() {
    // use bytemuck::cast;

    let strike : f32x8 = f32x8::splat(110.0);
    let years_to_expiry : f32x8 = f32x8::splat(0.5);
    let risk_free_rate : f32x8 = f32x8::splat(0.05);
    let dividend_yield : f32x8 = f32x8::splat(0.02);
    let volatility : f32x8 = f32x8::splat(0.25);
    let num_trials = 1000.0;
    let steps = 100.0;
    let now = std::time::Instant::now();
    let extra = f32x8::from([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);

    for i in (50..162).step_by(8) {
        let spot : f32x8 = f32x8::splat(i as f32) + extra;
        _ = monte_carlo_call_price_f32x8(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, steps, num_trials);
    }

    let duration = now.elapsed().as_millis();

    // let actual: [f32; 8] = cast(prices);

    println!("Time take [Fast Monte-carlo f32x8] {}ms", duration);
}

fn main() {
    let spot : f32 = 100.0;
    let strike : f32 = 110.0;
    let years_to_expiry = 0.5;
    let risk_free_rate : f32 = 0.05;
    let dividend_yield : f32 = 0.02;
    let volatility : f32 = 0.25;
    let num_trials : f32 = 1000.0;
    let steps : f32 = 100.0;

    println!("Black-scholes price: {}", black_scholes_call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield));
    println!("Monte-carlo price: {}", monte_carlo_call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, steps, num_trials));
}
