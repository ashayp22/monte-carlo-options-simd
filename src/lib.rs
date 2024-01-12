/*
TODO:
- SIMD randomization
- benchmark with different mc params

*/
#![feature(portable_simd)]

mod rand32x8;
pub mod bs;
pub mod mc;
pub mod mc32x8;
pub mod mcfast;
