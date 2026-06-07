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
pub use intmap::{IntScale, int_scale};
pub use lerp::*;
pub use math8::*;
pub use random::{RangeSample, Rng16};
pub use scale8::*;
pub use trig8::*;

// FastLED's `lib8tion.h` defines these as plain typedefs (`fract8`, `fract16`,
// `sfract8`, `accum88`, ...) — every numerator is just a bare integer, so a
// brightness byte and a scale factor are interchangeable to the C compiler
// (and to a Rust type alias, which is just a second name for the same type).
// Wrapping each in its own `#[repr(transparent)]` `Copy` newtype costs
// nothing at runtime — it's erased entirely by the optimizer — but lets the
// type checker catch `scale8(brightness, brightness_again)`-style mixups at
// the call site instead of producing a value that's silently the wrong shape.
macro_rules! fixed_point_numerator {
    ($(#[$meta:meta])* $name:ident($repr:ty)) => {
        $(#[$meta])*
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name(pub $repr);

        impl $name {
            #[doc = concat!("Wraps `numerator` as a [`", stringify!($name), "`].")]
            #[inline]
            pub const fn new(numerator: $repr) -> Self {
                Self(numerator)
            }

            /// Returns the underlying numerator.
            #[inline]
            pub const fn value(self) -> $repr {
                self.0
            }
        }

        impl From<$repr> for $name {
            #[inline]
            fn from(numerator: $repr) -> Self {
                Self(numerator)
            }
        }

        impl From<$name> for $repr {
            #[inline]
            fn from(wrapped: $name) -> Self {
                wrapped.0
            }
        }
    };
}

fixed_point_numerator! {
    /// 8-bit fixed-point fraction in the range `0/256 ..= 255/256`, stored as
    /// a numerator over 256. The scale-factor type for [`scale8`](scale8::scale8),
    /// [`lerp8by8`](lerp::lerp8by8) and friends — build one with `Fract8(n)`/
    /// [`Fract8::new`], or `n.into()`.
    Fract8(u8)
}

fixed_point_numerator! {
    /// 16-bit fixed-point fraction in the range `0/65536 ..= 65535/65536`,
    /// stored as a numerator over 65536. The scale-factor type for
    /// [`scale16`](scale8::scale16), [`lerp16by16`](lerp::lerp16by16) and
    /// friends.
    Fract16(u16)
}

fixed_point_numerator! {
    /// Signed 8-bit fixed-point fraction in the range `-1.0 ..= 1.0`,
    /// stored as a numerator over 128.
    SFract8(i8)
}

fixed_point_numerator! {
    /// 8.8 fixed-point accumulator: high byte is the integer part, low byte
    /// is the fractional part — e.g. a BPM of `120.5` is `Accum88((120 << 8) | 128)`.
    /// The format [`beat88`](beat::beat88)/[`beatsin88`](beat::beatsin88)
    /// expect their tempo in.
    Accum88(u16)
}
