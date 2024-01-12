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

pub fn black_scholes_call_price(
    spot: f32,
    strike: f32,
    volatility: f32,
    risk_free_rate: f32,
    years_to_expiry: f32,
    dividend_yield: f32
) -> f32 {
    let d1: f32 =
        ((spot / strike).ln() +
            (risk_free_rate - dividend_yield + (volatility * volatility) / 2.0) * years_to_expiry) /
        (volatility * years_to_expiry.sqrt());
    let d2: f32 = d1 - volatility * years_to_expiry.sqrt();

    let call: f32 =
        spot * (-dividend_yield * years_to_expiry).exp() * normal_cdf(d1) -
        strike * (-risk_free_rate * years_to_expiry).exp() * normal_cdf(d2);
    call
}
