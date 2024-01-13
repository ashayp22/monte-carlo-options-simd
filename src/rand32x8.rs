use wide::*;
use simd_rand::portable::*;

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
