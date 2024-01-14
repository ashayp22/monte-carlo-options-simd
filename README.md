# Monte Carlo Options Pricer with SIMD

Prices European options and calculates Greeks using Monte Carlo Simulations and achieves a **~9x** speed-up with SIMD operations compared to scalar operations. This library requires [nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html).

Available modules:

- [`mc_simd`] - pricing options with SIMD operations
  - [`mc_simd::call_price`] - calculate the price of a call option given strike, spot, risk-free rate, dividend, and time to expiry
  - [`mc_simd::put_price`] - calculate the price of a put option
  - [`mc_simd::call_price_av`] - apply antithetic variate method to reduce variance in simulated prices
  - [`mc_simd::call_delta`] - calculate Delta for call options
  - [`mc_simd::put_delta`] - calculate Delta for put options
  - [`mc_simd::gamma`] - calculate Gamma
  - [`mc_simd::vega`] - calculate Vega
  - [`mc_simd::call_rho`] - calculate Gamma for call options
  - [`mc_simd::put_rho`] - calculate Gamma for put options
  - [`mc_simd::call_theta`] - calculate Theta for call options
  - [`mc_simd::put_theta`] - calculate Theta for put options
- [`mc`] - pricing options with scalar operations, used to compare performance
  - [`mc::call_price`]
  - [`mc::put_price`]

# Background

Monte Carlo simulations are one of the most popular methods of [pricing financial options](https://www.tejwin.com/en/insight/options-pricing-with-monte-carlo-simulation/), especially when a closed form equation is not available. However, these simulations require a large number of trials to calculate the underlying's price over option's lifetime, which motivates the use of SIMD to speed up these calculations.

SIMD allows us to perform operations on multiple data with one instruction, which is especially helpful in the inner loop of the Monte Carlo simulation where we're doing the same set of operations `n` number of times.

So instead of this,

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

To calculate the Greeks, we get option prices using a Monte Carlo simulation and apply the finite difference method (central difference) to determine Delta, Gamma, Theta, Vega, and Rho.

# Usage

```toml
[dependencies]
monte_carlo_options_simd = { git = "https://github.com/ashayp22/monte-carlo-options-simd" }
```

```rust
use rand_core::{RngCore, SeedableRng};
use simd_rand::portable::*;
use monte_carlo_options_simd::*;
use wide::*;

fn main() {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let spot : f32 = 100.0;
    let strike : f32 = 110.0;
    let volatility : f32 = 0.25;
    let risk_free_rate : f32 = 0.05;
    let years_to_expiry : f32 = 0.5;
    let dividend_yield : f32 = 0.02;

    // Get the option price
    let call_option_price: f32 = mc_simd::call_price(spot, strike, volatility, risk_free_rate, years_to_expiry, dividend_yield, 100.0, 1000.0, &mut rng);
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

- `mc::call_price()`: ~415ms
- `mc_simd::call_price()`: ~46ms

For 112 spots, 10000 trials and 100 steps:

- `mc::call_price()`: ~4.15s
- `mc_simd::call_price()`: ~469ms

Comparing mc_simd (SIMD) to mc (scalar), we get a 9x improvement in speed.

# Resources

- [Math behind Monte Carlo simulations for pricing options](https://www.codearmo.com/blog/pricing-options-monte-carlo-simulation-python)
- [Another example of Monte Carlo simulations for pricing options](https://www.tejwin.com/en/insight/options-pricing-with-monte-carlo-simulation/)
- [Designing a SIMD Algorithm from Scratch](https://mcyoung.xyz/2023/11/27/simd-base64/)

# License

[GNU GPL v3](LICENSE)
