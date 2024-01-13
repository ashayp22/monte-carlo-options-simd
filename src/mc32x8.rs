use wide::*;
use simd_rand::portable::*;
use crate::rand32x8::get_rand_uniform_f32x8;
use rand_core::{ RngCore, SeedableRng };
use crate::bs::{ black_scholes_call_price, black_scholes_put_price };

pub fn call_price(
    spot: f32x8,
    strike: f32x8,
    volatility: f32x8,
    risk_free_rate: f32x8,
    years_to_expiry: f32x8,
    dividend_yield: f32x8,
    steps: f32,
    num_trials: f32,
    rng: &mut Xoshiro256PlusPlusX8
) -> f32x8 {
    let dt: f32x8 = years_to_expiry / steps;
    let nudt: f32x8 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32x8 = volatility * dt.sqrt();
    let nusidt: f32x8 = nudt + sidt;
    let half_steps: i32 = (steps as i32) / 2;
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let nusidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2) * nusidt;

    let zeros: f32x8 = f32x8::splat(0.0);
    let mut total_prices: f32x8 = f32x8::splat(0.0);

    for _ in 0..num_trials as i32 {
        let mut stock_price_mult = f32x8::splat(0.0);

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
            f32x8::mul_sub(spot, (stock_price_mult * nusidt_two_sqrt).exp(), strike),
            zeros
        );
    }

    let call_option_prices: f32x8 =
        (total_prices / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    call_option_prices
}

pub fn put_price(
    spot: f32x8,
    strike: f32x8,
    volatility: f32x8,
    risk_free_rate: f32x8,
    years_to_expiry: f32x8,
    dividend_yield: f32x8,
    steps: f32,
    num_trials: f32,
    rng: &mut Xoshiro256PlusPlusX8
) -> f32x8 {
    let dt: f32x8 = years_to_expiry / steps;
    let nudt: f32x8 = (risk_free_rate - dividend_yield - 0.5 * (volatility * volatility)) * dt;
    let sidt: f32x8 = volatility * dt.sqrt();
    let nusidt: f32x8 = nudt + sidt;
    let half_steps: i32 = (steps as i32) / 2;
    let two_pi = f32x8::splat(2.0 * std::f32::consts::PI);
    let nusidt_two_sqrt = f32x8::splat(std::f32::consts::SQRT_2) * nusidt;

    let zeros: f32x8 = f32x8::splat(0.0);
    let mut total_prices: f32x8 = f32x8::splat(0.0);

    for _ in 0..num_trials as i32 {
        let mut stock_price_mult = f32x8::splat(0.0);

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
            f32x8::mul_neg_add(spot, (stock_price_mult * nusidt_two_sqrt).exp(), strike),
            zeros
        );
    }

    let put_option_prices: f32x8 =
        (total_prices / num_trials) * (-risk_free_rate * years_to_expiry).exp();
    put_option_prices
}

#[test]
fn valid_price1() {
    use bytemuck::cast;

    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(100.0, 110.0, 0.25, 0.05, 0.5, 0.02);

    let price: [f32; 8] = cast(
        call_price(
            f32x8::splat(100.0),
            f32x8::splat(110.0),
            f32x8::splat(0.25),
            f32x8::splat(0.05),
            f32x8::splat(0.5),
            f32x8::splat(0.02),
            100.0,
            1000.0,
            &mut rng
        )
    );
    println!("mc32x8 1 {} vs {}", price[0], actual_price);
    assert_eq!(actual_price - 1.25 <= price[0] && price[0] <= actual_price + 1.25, true);
}

#[test]
fn valid_price2() {
    use bytemuck::cast;

    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(110.0, 110.0, 0.2, 0.05, 0.5, 0.05);

    let price: [f32; 8] = cast(
        call_price(
            f32x8::splat(110.0),
            f32x8::splat(110.0),
            f32x8::splat(0.2),
            f32x8::splat(0.05),
            f32x8::splat(0.5),
            f32x8::splat(0.05),
            100.0,
            1000.0,
            &mut rng
        )
    );
    println!("mc32x8 2 {} vs {}", price[0], actual_price);
    assert_eq!(actual_price - 1.25 <= price[0] && price[0] <= actual_price + 1.25, true);
}

#[test]
fn valid_price3() {
    use bytemuck::cast;

    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_call_price(115.0, 110.0, 0.2, 0.05, 0.5, 0.01);

    let price: [f32; 8] = cast(
        call_price(
            f32x8::splat(115.0),
            f32x8::splat(110.0),
            f32x8::splat(0.2),
            f32x8::splat(0.05),
            f32x8::splat(0.5),
            f32x8::splat(0.01),
            100.0,
            1000.0,
            &mut rng
        )
    );
    println!("mc32x8 3 {} vs {}", price[0], actual_price);
    assert_eq!(actual_price - 1.25 <= price[0] && price[0] <= actual_price + 1.25, true);
}

#[test]
fn valid_price_4() {
    use bytemuck::cast;

    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let actual_price = black_scholes_put_price(115.0, 110.0, 0.2, 0.05, 0.5, 0.01);

    let price: [f32; 8] = cast(
        put_price(
            f32x8::splat(115.0),
            f32x8::splat(110.0),
            f32x8::splat(0.2),
            f32x8::splat(0.05),
            f32x8::splat(0.5),
            f32x8::splat(0.01),
            100.0,
            1000.0,
            &mut rng
        )
    );
    println!("mc32x8 4 {} vs {}", price[0], actual_price);
    assert_eq!(actual_price - 1.25 <= price[0] && price[0] <= actual_price + 1.25, true);
}
