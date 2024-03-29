// Black-scholes pricer, used to test the Monte-carlo simulation pricers

fn erf(x: f32) -> f32 {
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

fn normal_cdf(x: f32) -> f32 {
    0.5 * (1.0 + erf(x / (2.0f32).sqrt()))
}

fn pdf(x: f32, mu: f32, sigma: f32) -> f32 {
    ((-1.0 * (x - mu) * (x - mu)) / (2.0 * sigma * sigma)).exp() /
        (sigma * (2.0 * 3.14159f32).sqrt())
}

fn d(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> (f32, f32) {
    let d1: f32 =
        ((spot / strike).ln() +
            (risk_free_rate - dividend_yield + (volatility * volatility) / 2.0) * years_to_expiry) /
        (volatility * years_to_expiry.sqrt());
    let d2: f32 = d1 - volatility * years_to_expiry.sqrt();
    (d1, d2)
}

pub(crate) fn call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);

    let call: f32 =
        spot * (-dividend_yield * years_to_expiry).exp() * normal_cdf(d1) -
        strike * (-risk_free_rate * years_to_expiry).exp() * normal_cdf(d2);
    call
}

pub(crate) fn put_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);

    let put: f32 =
        strike * (-risk_free_rate * years_to_expiry).exp() * (1.0 - normal_cdf(d2)) -
        spot * (-dividend_yield * years_to_expiry).exp() * (1.0 - normal_cdf(d1));
    put
}

pub(crate) fn gamma(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, _) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd1 = pdf(d1, 0.0, 1.0);
    ((-dividend_yield * years_to_expiry).exp() * nd1) / (spot * volatility * years_to_expiry.sqrt())
}

pub(crate) fn call_delta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, _) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd1 = normal_cdf(d1);
    (-dividend_yield * years_to_expiry).exp() * nd1
}

pub(crate) fn put_delta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, _) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd1 = normal_cdf(d1);
    (-dividend_yield * years_to_expiry).exp() * (nd1 - 1.0)
}

pub(crate) fn vega(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, _) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd1 = pdf(d1, 0.0, 1.0);
    ((-dividend_yield * years_to_expiry).exp() * nd1 * (spot * years_to_expiry.sqrt())) / 100.0
}

pub(crate) fn call_rho(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (_, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd2 = normal_cdf(d2);
    strike * years_to_expiry * (-risk_free_rate * years_to_expiry).exp() * nd2 / 100.0
}

pub(crate) fn put_rho(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (_, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let nd2 = normal_cdf(d2);
    strike * years_to_expiry * (-risk_free_rate * years_to_expiry).exp() * (nd2 - 1.0) / 100.0
}

pub(crate) fn call_theta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let result =
        (-(-dividend_yield * years_to_expiry).exp() * spot * pdf(d1, 0.0, 1.0) * volatility) /
        (2.0 * years_to_expiry.sqrt());
    let result_part1 = risk_free_rate * strike * (-risk_free_rate * years_to_expiry).exp();
    let result_part2 = dividend_yield * spot * (-dividend_yield * years_to_expiry).exp();
    result - result_part1 * normal_cdf(d2) + result_part2 * normal_cdf(d1)
}

pub(crate) fn put_theta(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let (d1, d2) = d(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield);
    let result =
        (-(-dividend_yield * years_to_expiry).exp() * spot * pdf(d1, 0.0, 1.0) * volatility) /
        (2.0 * years_to_expiry.sqrt());
    let result_part1 = risk_free_rate * strike * (-risk_free_rate * years_to_expiry).exp();
    let result_part2 = dividend_yield * spot * (-dividend_yield * years_to_expiry).exp();
    result + result_part1 * normal_cdf(-d2) - result_part2 * normal_cdf(-d1)
}