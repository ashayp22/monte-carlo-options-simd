#![feature(portable_simd)]

// SIMD PRNG
mod rand32x8;

// Black-scholes priceer, used to test Monte-carlo simulation pricers
mod bs;

// Monte-carlo simulation modules
pub mod mc;
pub mod mcfast;
