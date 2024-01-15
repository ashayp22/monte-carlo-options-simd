use wide::*;
use simd_rand::portable::*;
use rand_core::{RngCore, SeedableRng};

pub fn get_rand_uniform_f32x8(rng: &mut Xoshiro256PlusPlusX8) -> f32x8 {
    let rand_f64x8: [f64; 8] = rng.next_f64x8().to_array();

    // Cast required since simd_rand doesn't support f32x8 random number generation
    let random_mult = f32x8::from([
        rand_f64x8[0] as f32,
        rand_f64x8[1] as f32,
        rand_f64x8[2] as f32,
        rand_f64x8[3] as f32,
        rand_f64x8[4] as f32,
        rand_f64x8[5] as f32,
        rand_f64x8[6] as f32,
        rand_f64x8[7] as f32,
    ]);

    random_mult
}

fn test_uniform_distribution(samples: usize, diff_limit: f32) {
    let mut seed: Xoshiro256PlusPlusX8Seed = Default::default();
    rand::thread_rng().fill_bytes(&mut *seed);
    let mut rng: Xoshiro256PlusPlusX8 = Xoshiro256PlusPlusX8::from_seed(seed);

    let mut dist: Vec<f32> = Vec::with_capacity(samples);

    let mut sum = 0.0;
    for _ in 0..samples {
        let value = (get_rand_uniform_f32x8(&mut rng).to_array())[0];
        sum = sum + value;
        dist.push(value);
    }

    let mean: f32 = sum / (samples as f32);

    let mut squared_diffs = 0.0;
    for n in dist {
        let diff = (n - mean).powi(2);
        squared_diffs += diff;
    }

    // In uniform distribution, where the interval is a to b

    // the mean should be: μ = (a + b) / 2
    let expected_mean = 0.5;
    let mean_difference = (mean - expected_mean).abs();

    let variance = squared_diffs / (samples as f32);
    // the variance should be: σ2 = (b – a)2 / 12
    let expected_variance = (1.0) / 12.0;
    let variance_difference = (variance - expected_variance).abs();

    let stddev = variance.sqrt();
    // The standard deviation should be: σ = √σ2
    let expected_stddev = expected_variance.sqrt();
    let stddev_difference = (stddev - expected_stddev).abs();

    // If any of these metrics deviate by DIFF_LIMIT or more,
    // we should fail the test
    assert!(mean_difference <= diff_limit,"Mean difference was more than {diff_limit:.5}: {mean_difference:.5}. Expected mean: {expected_mean:.6}, actual mean: {mean:.6}");
    assert!(variance_difference <= diff_limit, "Variance difference was more than {diff_limit:.5}: {variance_difference:.5}. Expected variance: {expected_variance:.6}, actual variance: {variance:.6}");
    assert!(stddev_difference <= diff_limit, "Std deviation difference was more than {diff_limit:.5}: {stddev_difference:.5}. Expected std deviation: {expected_stddev:.6}, actual std deviation: {stddev:.6}");
}

#[test]
fn test_uniform_distribution_100() {
    test_uniform_distribution(100, 0.03);
}

#[test]
fn test_uniform_distribution_1000() {
    test_uniform_distribution(1000, 0.015);
}

#[test]
fn test_uniform_distribution_100000() {
    test_uniform_distribution(100000, 0.001);
}