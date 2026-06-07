//! `Qfx<F>` — a small fixed-point "scale factor" type for multiplying
//! integers by a non-integer ratio without floating point.
//!
//! Direct port of FastLED's `qfx<T, F, I>` template. The original packs an
//! integer part (`I` bits) and fractional part (`F` bits) into adjacent
//! bitfields of a storage type `T`; since the storage layout has no bearing
//! on the arithmetic (only the fractional bit-width `F` — used as the
//! right-shift amount — does), this port keeps the parts as plain fields and
//! generalizes over `F` with a `const` generic, which is both simpler and
//! removes the storage-width ceiling the original imposed on the integer
//! part.

use core::ops::Mul;

/// A fixed-point ratio `integer + fraction / 2^F`, usable as a multiplier for
/// integers via the [`Mul`] operator (in either order: `q * v` or `v * q`).
///
/// `F` is the number of fractional bits — equivalently, `fraction` is a
/// numerator over `2^F`. Multiplying by a `Qfx<F>` computes
/// `v * integer + (v * fraction) >> F`, i.e. `v * (integer + fraction / 2^F)`
/// without ever forming a float.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qfx<const F: u32> {
    integer: u32,
    fraction: u32,
}

impl<const F: u32> Qfx<F> {
    /// Builds a `Qfx` directly from its integer and fractional parts.
    /// `fraction` is interpreted as a numerator over `2^F` (i.e. it should
    /// fit in `F` bits — values outside that range are accepted but will
    /// behave as if `> 1.0` was added to `integer`).
    #[inline]
    pub const fn from_parts(integer: u32, fraction: u32) -> Self {
        Self { integer, fraction }
    }

    /// Approximates a non-negative floating-point ratio as a `Qfx`, by
    /// truncating the integer part and quantizing the remainder to `F`
    /// fractional bits.
    #[inline]
    pub fn from_f32(value: f32) -> Self {
        let integer = value as u32;
        let fraction = ((value - integer as f32) * (1u32 << F) as f32) as u32;
        Self { integer, fraction }
    }

    /// The integer part.
    #[inline]
    pub const fn integer_part(self) -> u32 {
        self.integer
    }

    /// The fractional part, as a numerator over `2^F`.
    #[inline]
    pub const fn fraction_part(self) -> u32 {
        self.fraction
    }
}

macro_rules! impl_mul_unsigned {
    ($narrow:ty, $wide:ty) => {
        impl<const F: u32> Mul<$narrow> for Qfx<F> {
            type Output = $narrow;

            // The `>>` here rescales the fractional product back from `2^F`
            // fixed-point into the output's integer domain — it's the
            // defining operation of fixed-point multiplication, not a typo
            // for `*`/`+`.
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline]
            fn mul(self, v: $narrow) -> $narrow {
                let v = v as $wide;
                let whole = v.wrapping_mul(self.integer as $wide);
                let frac = (v.wrapping_mul(self.fraction as $wide)) >> F;
                whole.wrapping_add(frac) as $narrow
            }
        }

        impl<const F: u32> Mul<Qfx<F>> for $narrow {
            type Output = $narrow;

            #[inline]
            fn mul(self, q: Qfx<F>) -> $narrow {
                q * self
            }
        }
    };
}

macro_rules! impl_mul_signed {
    ($narrow:ty, $wide:ty) => {
        impl<const F: u32> Mul<$narrow> for Qfx<F> {
            type Output = $narrow;

            // See the unsigned impl above — the `>>` rescales the fixed-point
            // fractional product, it isn't a mistaken operator.
            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline]
            fn mul(self, v: $narrow) -> $narrow {
                let v = v as $wide;
                let whole = v.wrapping_mul(self.integer as $wide);
                let frac = (v.wrapping_mul(self.fraction as $wide)) >> F;
                whole.wrapping_add(frac) as $narrow
            }
        }

        impl<const F: u32> Mul<Qfx<F>> for $narrow {
            type Output = $narrow;

            #[inline]
            fn mul(self, q: Qfx<F>) -> $narrow {
                q * self
            }
        }
    };
}

impl_mul_unsigned!(u16, u32);
impl_mul_unsigned!(u32, u64);
impl_mul_signed!(i16, i32);
impl_mul_signed!(i32, i64);

// NOTE: FastLED names these `qfx<T, F, I>` — fractional bits *F* come before
// integer bits *I* in both the template parameter list and the `qFI` alias
// names below, which is easy to misread. Pulled directly from FastLED's
// `typedef qfx<u8,4,4> q44`, `typedef qfx<u8,6,2> q62`,
// `typedef qfx<u16,8,8> q88`, `typedef qfx<u16,12,4> q124`.

/// A "4.4" fixed-point ratio: 4 fractional bits, 4 integer bits.
pub type Q44 = Qfx<4>;
/// A "6.2" fixed-point ratio: 6 fractional bits, 2 integer bits.
pub type Q62 = Qfx<6>;
/// An "8.8" fixed-point ratio: 8 fractional bits, 8 integer bits.
pub type Q88 = Qfx<8>;
/// A "12.4" fixed-point ratio: 12 fractional bits, 4 integer bits.
pub type Q124 = Qfx<12>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplies_like_a_ratio() {
        // 1.5 represented in Q8.8 (fraction = 0.5 * 256 = 128)
        let one_point_five = Q88::from_parts(1, 128);
        assert_eq!(one_point_five * 100u32, 150);
        assert_eq!(100u32 * one_point_five, 150);
        assert_eq!(one_point_five * 100u16, 150);
        assert_eq!(one_point_five * 100i32, 150);
        assert_eq!(one_point_five * 100i16, 150);
    }

    #[test]
    fn integer_only_ratio_is_plain_multiplication() {
        let three = Q44::from_parts(3, 0);
        for v in 0u16..=1000 {
            assert_eq!(three * v, v.wrapping_mul(3));
        }
    }

    #[test]
    fn from_f32_quantizes_into_parts() {
        let q = Q88::from_f32(2.5);
        assert_eq!(q.integer_part(), 2);
        assert_eq!(q.fraction_part(), 128); // 0.5 * 256
        assert_eq!(q * 4u32, 10);
    }
}
