use wide::*;
use simd_rand::portable::*;
use crate::rand32x8::get_rand_uniform_f32x8;
use rand_core::{ RngCore, SeedableRng };
use crate::bs;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

#[inline(always)]
fn speed_update(two_pi: f32x8, stock_price_mult: f32x8, rng: &mut Xoshiro256PlusPlusX8) -> f32x8 {
    // Simulate two steps in time in order to have clean, vertical f32x8 operations
    let first_rand: f32x8 = get_rand_uniform_f32x8(rng);
    let second_rand: f32x8 = get_rand_uniform_f32x8(rng);

    // The random f32x8 are uniformly distributed, so we have to
    // apply the Box-Muller transform to make them normally distributed
    let (sin_rand, cos_rand) = f32x8::sin_cos(two_pi * second_rand);

    // Only necessary operations are kept in the innermost loop
    f32x8::mul_add((-first_rand.ln()).sqrt(), sin_rand + cos_rand, stock_price_mult)
}

/* Below are the monte carlo simulation pricers */

fn monte_carlo_pricing(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    call_mult: f32 // 1.0 if pricing a call option, -1.0 if pricing a put option
) -> f32 {
    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();

    let two_pi: f32x8 = f32x8::splat(2.0 * std::f32::consts::PI);
    let strike_f32x8 = f32x8::splat(call_mult * strike);
    let spot_f32x8 = f32x8::splat(call_mult * spot);
    let zeros: f32x8 = f32x8::splat(0.0);
    let nudt_f32x8: f32x8 = f32x8::splat(steps * nudt); // multiply by steps since nudt appears n times in the inner most loop
    let sidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2 * sidt); // take the sqrt(2) out of the box muller transform

    let half_steps: i32 = (steps as i32) / 2;

    let total_prices: f32x8 = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            f32x8::fast_max(
                f32x8::mul_sub(
                    spot_f32x8,
                    f32x8::mul_add(stock_price_mult, sidt_two_sqrt, nudt_f32x8).exp(),
                    strike_f32x8
                ),
                zeros
            )
        })
        .reduce(
            || f32x8::splat(0.0),
            |a, b| a + b
        );

    let call_option_price: f32 =
        (total_prices.reduce_add() / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

// Apply the antithetic variate method to reduce variance in simulated prices
fn monte_carlo_av_pricing(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    call_mult: f32 // 1.0 if pricing a call option, -1.0 if pricing a put option
) -> f32 {
    let dt: f32 = years_to_expiry / steps;
    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();

    let nudt_f32x8: f32x8 = f32x8::splat(steps * nudt);
    let strike_f32x8 = f32x8::splat(call_mult * strike);
    let spot_f32x8 = f32x8::splat(call_mult * spot);
    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let sidt_two_sqrt_plus = f32x8::splat(std::f32::consts::SQRT_2 * sidt);
    let sidt_two_sqrt_neg = f32x8::splat(std::f32::consts::SQRT_2 * -sidt);

    let half_steps: i32 = (steps as i32) / 2;

    let (total_prices_pos, total_prices_neg) = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            (
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt_plus, nudt_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt_neg, nudt_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
            )
        })
        .reduce(
            || (f32x8::splat(0.0), f32x8::splat(0.0)),
            |(a, b), (c, d)| (a + c, b + d)
        );

    let call_option_price: f32 =
        ((0.5 * (total_prices_pos.reduce_add() + total_prices_neg.reduce_add())) / num_trials) *
        (-risk_free_rate * years_to_expiry).exp();
    call_option_price
}

// Price three options with spots of spot, spot - delta_spot, and spot + delta_spot
fn monte_carlo_spot_pricing(
    spot: f32,
    delta_spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    call_mult: f32 // 1.0 if pricing a call, -1.0 if pricing a put
) -> (f32, f32, f32) {
    let dt: f32 = years_to_expiry / steps;

    let nudt: f32 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();

    let nudt_f32x8: f32x8 = f32x8::splat(steps * nudt);
    let strike_f32x8 = f32x8::splat(strike * call_mult);
    let spot_f32x8 = f32x8::splat(call_mult * spot);
    let spot_minus_f32x8 = f32x8::splat(call_mult * (spot - delta_spot));
    let spot_plus_f32x8 = f32x8::splat(call_mult * (spot + delta_spot));

    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let sidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2 * sidt);

    let half_steps: i32 = (steps as i32) / 2;

    // Calculate three different three stock paths to find the Greeks delta and gamma
    let (total, total_plus, total_minus) = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            let stock_price_mult_exp = f32x8
                ::mul_add(stock_price_mult, sidt_two_sqrt, nudt_f32x8)
                .exp();

            (
                f32x8::fast_max(
                    f32x8::mul_sub(spot_f32x8, stock_price_mult_exp, strike_f32x8),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(spot_plus_f32x8, stock_price_mult_exp, strike_f32x8),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(spot_minus_f32x8, stock_price_mult_exp, strike_f32x8),
                    zeros
                ),
            )
        })
        .reduce(
            || (f32x8::splat(0.0), f32x8::splat(0.0), f32x8::splat(0.0)),
            |(a, b, c), (d, e, f)| (a + d, b + e, c + f)
        );

    let final_mult = (-risk_free_rate * years_to_expiry).exp() / num_trials;

    (
        total_minus.reduce_add() * final_mult,
        total.reduce_add() * final_mult,
        total_plus.reduce_add() * final_mult,
    )
}

// Price two options with volatility+delta_volatility, volatility-delta_volatility
fn monte_carlo_volatility_pricing(
    spot: f32,
    strike: f32,
    volatility: f32,
    delta_volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> (f32, f32) {
    let dt: f32 = years_to_expiry / steps;
    let volatility_plus = volatility + delta_volatility;
    let volatility_minus = volatility - delta_volatility;

    let nudt_plus: f32 =
        (risk_free_rate - dividend_yield - 0.5 * (volatility_plus * volatility_plus)) * dt;
    let nudt_minus: f32 =
        (risk_free_rate - dividend_yield - 0.5 * (volatility_minus * volatility_minus)) * dt;
    let sidt_plus: f32 = volatility_plus * dt.sqrt();
    let sidt_minus: f32 = volatility_minus * dt.sqrt();

    let strike_f32x8 = f32x8::splat(strike);
    let spot_f32x8 = f32x8::splat(spot);

    let nudt_plus_f32x8: f32x8 = f32x8::splat(steps * nudt_plus);
    let nudt_minus_f32x8: f32x8 = f32x8::splat(steps * nudt_minus);

    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let sidt_two_sqrt_plus = f32x8::splat(std::f32::consts::SQRT_2 * sidt_plus);
    let sidt_two_sqrt_minus = f32x8::splat(std::f32::consts::SQRT_2 * sidt_minus);

    let half_steps: i32 = (steps as i32) / 2;

    // Calculate two different three stock paths to find the Greek vega
    let (total_plus, total_minus) = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            (
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt_plus, nudt_plus_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8
                            ::mul_add(stock_price_mult, sidt_two_sqrt_minus, nudt_minus_f32x8)
                            .exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
            )
        })
        .reduce(
            || (f32x8::splat(0.0), f32x8::splat(0.0)),
            |(a, b), (c, d)| (a + c, b + d)
        );

    let final_mult = (-risk_free_rate * years_to_expiry).exp() / num_trials;

    (total_minus.reduce_add() * final_mult, total_plus.reduce_add() * final_mult)
}

// Price two options with interest rates + and - delta_risk_free_rate
fn monte_carlo_interest_pricing(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    delta_risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    call_mult: f32 // 1.0 if pricing a call, -1.0 if pricing a put
) -> (f32, f32) {
    let dt: f32 = years_to_expiry / steps;
    let rfr_plus = risk_free_rate + delta_risk_free_rate;
    let rfr_minus = risk_free_rate - delta_risk_free_rate;

    let nudt_plus: f32 = (rfr_plus - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let nudt_minus: f32 = (rfr_minus - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32 = volatility * dt.sqrt();

    let nudt_plus_f32x8: f32x8 = f32x8::splat(steps * nudt_plus);
    let nudt_minus_f32x8: f32x8 = f32x8::splat(steps * nudt_minus);

    let strike_f32x8 = f32x8::splat(strike * call_mult);
    let spot_f32x8 = f32x8::splat(spot * call_mult);

    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let sidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2 * sidt);

    let half_steps: i32 = (steps as i32) / 2;

    // Calculate two different three stock paths to find the Greek rho
    let (total_plus, total_minus) = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            (
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt, nudt_plus_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt, nudt_minus_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
            )
        })
        .reduce(
            || (f32x8::splat(0.0), f32x8::splat(0.0)),
            |(a, b), (c, d)| (a + c, b + d)
        );

    (
        (total_minus.reduce_add() * (-rfr_minus * years_to_expiry).exp()) / num_trials,
        (total_plus.reduce_add() * (-rfr_plus * years_to_expiry).exp()) / num_trials,
    )
}

// Price two options with years_to_expiry + and - delta_years_to_expiry
fn monte_carlo_time_pricing(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    delta_years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32,
    call_mult: f32 // 1.0 if pricing a call, -1.0 if pricing a put
) -> (f32, f32) {
    let time_plus = years_to_expiry + delta_years_to_expiry;
    let time_minus = years_to_expiry - delta_years_to_expiry;

    let dt_plus: f32 = time_plus / steps;
    let dt_minus: f32 = time_minus / steps;

    let nudt_plus: f32 =
        (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt_plus;
    let nudt_minus: f32 =
        (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt_minus;
    let sidt_plus: f32 = volatility * dt_plus.sqrt();
    let sidt_minus: f32 = volatility * dt_minus.sqrt();

    let strike_f32x8 = f32x8::splat(strike * call_mult);
    let spot_f32x8 = f32x8::splat(spot * call_mult);

    let nudt_plus_f32x8: f32x8 = f32x8::splat(steps * nudt_plus);
    let nudt_minus_f32x8: f32x8 = f32x8::splat(steps * nudt_minus);

    let zeros: f32x8 = f32x8::splat(0.0);
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let sidt_two_sqrt_plus = f32x8::splat(std::f32::consts::SQRT_2 * sidt_plus);
    let sidt_two_sqrt_minus = f32x8::splat(std::f32::consts::SQRT_2 * sidt_minus);

    let half_steps: i32 = (steps as i32) / 2;

    // Calculate two different three stock paths to find the Greek vega
    let (total_plus, total_minus) = (0..(num_trials as i32) / 8)
        .into_par_iter()
        .map(|_| {
            // Get the random number generator for this thread
            let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
            rand::thread_rng().fill_bytes(&mut *seed);
            let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

            let mut stock_price_mult: f32x8 = f32x8::splat(0.0);

            for _ in 0..half_steps {
                stock_price_mult = speed_update(two_pi, stock_price_mult, &mut rng);
            }

            (
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8::mul_add(stock_price_mult, sidt_two_sqrt_plus, nudt_plus_f32x8).exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
                f32x8::fast_max(
                    f32x8::mul_sub(
                        spot_f32x8,
                        f32x8
                            ::mul_add(stock_price_mult, sidt_two_sqrt_minus, nudt_minus_f32x8)
                            .exp(),
                        strike_f32x8
                    ),
                    zeros
                ),
            )
        })
        .reduce(
            || (f32x8::splat(0.0), f32x8::splat(0.0)),
            |(a, b), (c, d)| (a + c, b + d)
        );

    (
        (total_minus.reduce_add() * (-risk_free_rate * time_minus).exp()) / num_trials,
        (total_plus.reduce_add() * (-risk_free_rate * time_plus).exp()) / num_trials,
    )
}

/* Below are the methods for getting prices and Greeks */

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
    monte_carlo_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    )
}

pub fn put_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    monte_carlo_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        -1.0
    )
}

pub fn call_price_av(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    monte_carlo_av_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    )
}

pub fn put_price_av(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    monte_carlo_av_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        -1.0
    )
}

pub fn call_delta(
    spot: f32,
    delta_spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, _, price_plus) = monte_carlo_spot_pricing(
        spot,
        delta_spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    );
    return (price_plus - price_minus) / (2.0 * delta_spot);
}

pub fn put_delta(
    spot: f32,
    delta_spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, _, price_plus) = monte_carlo_spot_pricing(
        spot,
        delta_spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        -1.0
    );
    return (price_plus - price_minus) / (2.0 * delta_spot);
}

pub fn gamma(
    spot: f32,
    delta_spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price, price_plus) = monte_carlo_spot_pricing(
        spot,
        delta_spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    );
    return (price_plus - 2.0 * price + price_minus) / (delta_spot * delta_spot);
}

pub fn vega(
    spot: f32,
    strike: f32,
    volatility: f32,
    delta_volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price_plus) = monte_carlo_volatility_pricing(
        spot,
        strike,
        volatility,
        delta_volatility,
        risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials
    );
    // Multiplied by 200.0 since we care about a change in 1% of the volatility
    (price_plus - price_minus) / (200.0 * delta_volatility)
}

pub fn call_rho(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    delta_risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price_plus) = monte_carlo_interest_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        delta_risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    );

    // Multiplied by 200.0 since we care about a change in 1% of the interest rate
    (price_plus - price_minus) / (200.0 * delta_risk_free_rate)
}

pub fn put_rho(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    delta_risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price_plus) = monte_carlo_interest_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        delta_risk_free_rate,
        years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        -1.0
    );
    // Multiplied by 200.0 since we care about a change in 1% of the interest rate
    (price_plus - price_minus) / (200.0 * delta_risk_free_rate)
}

pub fn call_theta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    delta_years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price_plus) = monte_carlo_time_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        delta_years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        1.0
    );
    // price_minus comes first because it represents a future point in time compared to price_plus
    (price_minus - price_plus) / (2.0 * delta_years_to_expiry)
}

pub fn put_theta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    delta_years_to_expiry: f32,
    dividend_yield: f32,
    steps: f32,
    num_trials: f32
) -> f32 {
    let (price_minus, price_plus) = monte_carlo_time_pricing(
        spot,
        strike,
        volatility,
        risk_free_rate,
        years_to_expiry,
        delta_years_to_expiry,
        dividend_yield,
        steps,
        num_trials,
        -1.0
    );
    (price_minus - price_plus) / (2.0 * delta_years_to_expiry)
}

#[test]
fn valid_price1() {
    let actual_price = bs::call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);
    let price = call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mc_simd 1 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.25, true);
}

#[test]
fn valid_price2() {
    let actual_price = bs::call_price(90.0, 110.0, 0.2, 0.05, 1.0, 0.02);
    let price = call_price(90.0, 110.0, 0.2, 0.05, 1.0, 0.02, 100.0, 10000.0);
    println!("mc_simd 2 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.25, true);
}

#[test]
fn valid_price3() {
    let actual_price = bs::call_price(130.0, 120.0, 0.25, 0.05, 0.5, 0.02);
    let price = call_price(130.0, 120.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mc_simd 3 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.25, true);
}

#[test]
fn valid_price_4() {
    let actual_price = bs::put_price(300.0, 270.0, 0.2, 0.09, 1.0, 0.0);
    let price = put_price(300.0, 270.0, 0.2, 0.09, 1.0, 0.0, 100.0, 10000.0);
    println!("mc_simd 4 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.0, true);
}

#[test]
fn valid_price_av1() {
    let actual_price = bs::call_price(112.0, 110.0, 0.2, 0.05, 1.0, 0.02);
    let price = call_price_av(112.0, 110.0, 0.2, 0.05, 1.0, 0.02, 100.0, 10000.0);
    println!("mc_simd av1 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.0, true);
}

#[test]
fn valid_price_av2() {
    let actual_price = bs::call_price(112.0, 110.0, 0.14, 0.12, 1.0, 0.02);
    let price = call_price_av(112.0, 110.0, 0.14, 0.12, 1.0, 0.02, 100.0, 10000.0);
    println!("mc_simd av2 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.0, true);
}

#[test]
fn valid_price_av3() {
    let actual_price = bs::put_price(112.0, 110.0, 0.14, 0.12, 1.0, 0.02);
    let price = put_price_av(112.0, 110.0, 0.14, 0.12, 1.0, 0.02, 100.0, 10000.0);
    println!("mc_simd av3 {} vs {}", price, actual_price);
    assert_eq!((actual_price - price).abs() <= 1.0, true);
}

#[test]
fn valid_call_delta() {
    let actual_delta = bs::call_delta(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);
    let delta = call_delta(100.0, 0.001, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mc_simd call delta {} vs {}", delta, actual_delta);
    assert_eq!((delta - actual_delta).abs() < 0.05, true);
}

#[test]
fn valid_put_delta() {
    let actual_delta = bs::put_delta(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);
    let delta = put_delta(100.0, 0.001, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 1000.0);
    println!("mc_simd put delta {} vs {}", delta, actual_delta);
    assert_eq!((delta - actual_delta).abs() < 0.05, true);
}

#[test]
fn valid_gamma() {
    let actual_gamma = bs::gamma(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);
    let mc_gamma = gamma(100.0, 0.01, 110.0, 0.25, 0.05, 0.5, 0.02, 100.0, 10000.0);
    println!("mc_simd gamma {} vs {}", mc_gamma, actual_gamma);
    assert_eq!((mc_gamma - actual_gamma).abs() < 0.05, true);
}

#[test]
fn valid_vega() {
    let actual_gamma = bs::vega(100.0, 110.0, 0.1, 0.05, 0.5, 0.02);
    let mc_gamma = vega(100.0, 110.0, 0.1, 0.01, 0.05, 0.5, 0.02, 100.0, 10000.0);
    println!("mc_simd vega {} vs {}", mc_gamma, actual_gamma);
    assert_eq!((mc_gamma - actual_gamma).abs() < 0.05, true);
}

#[test]
fn valid_call_rho() {
    let actual_rho = bs::call_rho(130.0, 120.0, 0.25, 0.05, 0.5, 0.02);
    let rho = call_rho(130.0, 120.0, 0.25, 0.05, 0.01, 0.5, 0.02, 100.0, 10000.0);
    println!("mc_simd call rho {} vs {}", rho, actual_rho);
    assert_eq!((rho - actual_rho).abs() < 0.05, true);
}

#[test]
fn valid_put_rho() {
    let actual_rho = bs::put_rho(110.0, 120.0, 0.1, 0.1, 0.5, 0.02);
    let rho = put_rho(110.0, 120.0, 0.1, 0.1, 0.01, 0.5, 0.02, 100.0, 10000.0);
    println!("mc_simd put rho {} vs {}", rho, actual_rho);
    assert_eq!((rho - actual_rho).abs() < 0.05, true);
}

#[test]
fn valid_call_theta() {
    let actual_theta = bs::call_theta(110.0, 120.0, 0.1, 0.1, 0.5, 0.02);
    let theta = call_theta(110.0, 120.0, 0.1, 0.1, 0.5, 0.001, 0.02, 100.0, 10000.0);
    println!("mc_simd call theta {} vs {}", theta, actual_theta);
    assert_eq!((theta - actual_theta).abs() < 0.1, true);
}

#[test]
fn valid_put_theta() {
    let actual_theta = bs::put_theta(110.0, 120.0, 0.1, 0.1, 0.5, 0.02);
    let theta = put_theta(110.0, 120.0, 0.1, 0.1, 0.5, 0.001, 0.02, 100.0, 10000.0);
    println!("mc_simd put theta {} vs {}", theta, actual_theta);
    assert_eq!((theta - actual_theta).abs() < 0.1, true);
}

#[test]
fn valid_put_theta2() {
    let actual_theta = bs::put_theta(130.0, 120.0, 0.1, 0.1, 0.5, 0.02);
    let theta = put_theta(130.0, 120.0, 0.1, 0.1, 0.5, 0.001, 0.02, 100.0, 10000.0);
    println!("mc_simd put theta 2 {} vs {}", theta, actual_theta);
    assert_eq!((theta - actual_theta).abs() < 0.1, true);
}
