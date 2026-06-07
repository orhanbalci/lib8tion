//! `lib8tion` — fast 8-/16-bit fixed-point math for `no_std` embedded targets.
//!
//! This is a Rust port of the math primitives from
//! [FastLED's `lib8tion`](https://github.com/FastLED/FastLED/tree/master/src/lib8tion):
//! saturating/wrapping byte arithmetic, scaling & dimming, fast trigonometric
//! approximations, a small fast PRNG, integer range mapping and simple
//! fixed-point types. All functions are pure integer math with defined
//! overflow behavior (saturating, wrapping or truncating, matching the
//! semantics of the original C implementation) — nothing here panics or
//! allocates.
#![no_std]
#![forbid(unsafe_code)]

pub mod beat;
pub mod ease;
pub mod fixed_point;
pub mod intmap;
pub mod lerp;
pub mod math8;
pub mod random;
pub mod scale8;
pub mod trig8;

pub use beat::*;
pub use ease::*;
pub use fixed_point::{Q44, Q62, Q88, Q124, Qfx};
pub use intmap::int_scale;
pub use lerp::*;
pub use math8::*;
pub use random::Rng16;
pub use scale8::*;
pub use trig8::*;

/// 8-bit fixed-point fraction in the range `0/256 ..= 255/256`, stored as a
/// numerator over 256. Used as a scale factor by [`scale8`](scale8::scale8) and friends.
pub type Fract8 = u8;

/// 16-bit fixed-point fraction in the range `0/65536 ..= 65535/65536`, stored
/// as a numerator over 65536. Used as a scale factor by [`scale16`](scale8::scale16).
pub type Fract16 = u16;

/// Signed 8-bit fixed-point fraction in the range `-1.0 ..= 1.0`.
pub type SFract8 = i8;

/// 8.8 fixed-point accumulator: high byte is the integer part, low byte is
/// the fractional part (e.g. used to represent BPM values).
pub type Accum88 = u16;
