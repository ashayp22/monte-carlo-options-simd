# Monte Carlo Options Pricer with SIMD

Prices European options using Monte Carlo Simulations and achieves a ~7x speed-up with SIMD operations compared to scalar operations. This library requires [nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html).

Available modules:

- [`mc`] - pricing 1 option with scalar operations
  - [`mc::call_price`]
  - [`mc::put_price`]
- [`mcfast`] - pricing 1 option with SIMD operations
  - [`mcfast::call_price`]
  - [`mcfast::call_price_AV`] - apply antithetic variate method to reduce variance in simulated prices
  - [`mcfast::put_price`]
- [`mc32x8`] - pricing 8 options with SIMD operations
  - [`mc32x8::call_price`]
  - [`mc32x8::put_price`]

# Background

Monte Carlo simulations are one of the most popular methods of [pricing financial options](https://www.tejwin.com/en/insight/options-pricing-with-monte-carlo-simulation/), especially when a closed form equation is not available. However, these simulations require a large number of trials to calculate the underlying's price over option's lifetime, which motivates the use of SIMD to speed up these calculations.

SIMD allows us to perform operations on multiple data with one instruction, which is especially helpful in the inner loop of the Monte Carlo simulation where we're doing the same set of operations `n` number of times.

So instead of this:

```rust
for _ in 0..num_trials {
    let mut count = 0;
    ...
    for step in 0..num_steps {
        count += step.ln();
        ...
    }
}
```

we can do something like this, arriving at a ~8x speed-up.

```rust
for _ in 0..num_trials/8 {
    let mut count = f32x8::splat(0.0);
    for step in 0..num_steps {
        count += f32x8::splat(step).ln();
        ...
    }
}
```

We make use of [wide](https://docs.rs/wide/latest/wide/) for SIMD-compatible data types and [simd-rand](https://github.com/ashayp22/simd-rand) for vectorized random number generation to get 6-8x speed-up when generating random numbers inside the inner-most loop of the Monte Carlo simulation as opposed to individually generating 8 random, normally distributed numbers inside the inner-most loop.

The final trick is to reduce the number of math operations inside the inner-most loop of the Monte Carlo simulation and use Fused Multiply-Add.

# Usage

```toml
[dependencies]
mc_options_simd = { git = "https://github.com/ashayp22/monte-carlo-options-simd" }
```

```rust
use rand_core::{RngCore, SeedableRng};
use simd_rand::portable::*;
use mc_options_simd::*;
use wide::*;

fn main() {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let spot : f32 = 50;
    let strike : f32 = 110.0;
    let years_to_expiry : f32 = 0.5;
    let risk_free_rate : f32 = 0.05;
    let dividend_yield : f32 = 0.02;
    let volatility : f32 = 0.25;

    // Get a single call option price
    let call_option_price = mcfast::call_price(spot, strike, years_to_expiry, dividend_yield, volatility, 100.0, 1000.0, &mut rng);

    let spot_increment : f32x8 = f32x8::from([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    let spot : f32x8 = f32x8::splat(spot) + spot_increment;
    let strike_f32x8 : f32x8 = f32x8::splat(strike);
    let years_to_expiry_f32x8 : f32x8 = f32x8::splat(years_to_expiry);
    let risk_free_rate_f32x8 : f32x8 = f32x8::splat(RISK_FREE_RATE);
    let dividend_yield_f32x8 : f32x8 = f32x8::splat(risk_free_rate);
    let volatility_f32x8 : f32x8 = f32x8::splat(volatility);

    // Get 8 put option prices
    let call_option_price_f32x8 = mc32x8::put_price(spot, strike_f32x8, volatility_f32x8, risk_free_rate_f32x8, years_to_expiry_f32x8, dividend_yield_f32x8, 100.0, 1000.0, &mut rng);
}
```

You may need to set RUSTFLAGS to get the “the best” code possible for the machine that you’re working on.

```sh
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
cargo run
...
```

# Performance

Calculated on a Macbook M1:

For 112 spots, 1000 trials and 100 steps:

- `mc::call_price()`: ~300ms
- `mcfast::call_price()`: ~44ms
- `mc32x8::call_price()`: ~46ms

For 112 spots, 10000 trials and 100 steps:

- `mc::call_price()`: ~3.03s
- `mcfast::call_price()`: ~442ms
- `mc32x8::call_price()`: ~463ms

Comparing mcfast and mc32x8 (SIMD) to mc (scalar), we get a 6.5-7x improvement in speed.

# Resources

- [Math behind Monte Carlo simulations for pricing options](https://www.codearmo.com/blog/pricing-options-monte-carlo-simulation-python)
- [Another example of Monte Carlo simulations for pricing options](https://www.tejwin.com/en/insight/options-pricing-with-monte-carlo-simulation/)
- [Designing a SIMD Algorithm from Scratch](https://mcyoung.xyz/2023/11/27/simd-base64/)

# License

[GNU GPL v3](LICENSE).
