//! Fast 8-/16-bit integer arithmetic: saturating & wrapping add/sub,
//! averages, multiplication, blending, modulo and integer square roots.
//!
//! These are direct ports of the portable-C implementations in FastLED's
//! `math8.h` (the AVR/ARM assembly fast paths are dropped — Rust's codegen
//! produces equivalent or better code for these tiny integer ops from the
//! plain formulas below).

use crate::intmap::int_scale;

/// Add two bytes, saturating at `0xFF` (`u8::MAX`) on overflow.
#[inline(always)]
pub const fn qadd8(i: u8, j: u8) -> u8 {
    i.saturating_add(j)
}

/// Add two signed bytes, saturating at `0x7F` / `-0x80`.
#[inline(always)]
pub const fn qadd7(i: i8, j: i8) -> i8 {
    i.saturating_add(j)
}

/// Subtract `j` from `i`, saturating at `0x00` on underflow.
#[inline(always)]
pub const fn qsub8(i: u8, j: u8) -> u8 {
    i.saturating_sub(j)
}

/// Add two bytes, with 8-bit (wrapping) result.
#[inline(always)]
pub const fn add8(i: u8, j: u8) -> u8 {
    i.wrapping_add(j)
}

/// Add a byte to a 16-bit value, with 16-bit (wrapping) result.
#[inline(always)]
pub const fn add8to16(i: u8, j: u16) -> u16 {
    (i as u16).wrapping_add(j)
}

/// Subtract one byte from another, with 8-bit (wrapping) result.
#[inline(always)]
pub const fn sub8(i: u8, j: u8) -> u8 {
    i.wrapping_sub(j)
}

/// Average of two unsigned bytes: `(i + j) / 2`, rounded down.
#[inline(always)]
pub const fn avg8(i: u8, j: u8) -> u8 {
    ((i as u16 + j as u16) >> 1) as u8
}

/// Average of two unsigned bytes, rounded up: `(i + j + 1) / 2`.
#[inline(always)]
pub const fn avg8r(i: u8, j: u8) -> u8 {
    ((i as u16 + j as u16 + 1) >> 1) as u8
}

/// Average of two unsigned 16-bit values: `(i + j) / 2`, rounded down.
#[inline(always)]
pub const fn avg16(i: u16, j: u16) -> u16 {
    ((i as u32 + j as u32) >> 1) as u16
}

/// Average of two unsigned 16-bit values, rounded up: `(i + j + 1) / 2`.
#[inline(always)]
pub const fn avg16r(i: u16, j: u16) -> u16 {
    ((i as u32 + j as u32 + 1) >> 1) as u16
}

/// Average of two signed 7-bit values held in `i8`.
///
/// Matches FastLED's `avg7`, which computes the average without overflow by
/// averaging the halves and folding in the lost low bit:
/// `(i >> 1) + (j >> 1) + (i & 1)`.
#[inline(always)]
pub const fn avg7(i: i8, j: i8) -> i8 {
    (i >> 1) + (j >> 1) + (i & 0x1)
}

/// Average of two signed 15-bit values held in `i16`.
///
/// See [`avg7`] — same trick, scaled up to 16 bits.
#[inline(always)]
pub const fn avg15(i: i16, j: i16) -> i16 {
    (i >> 1) + (j >> 1) + (i & 0x1)
}

/// 8x8-bit multiplication, keeping only the low 8 bits of the result
/// (i.e. `(i * j) mod 256`).
#[inline(always)]
pub const fn mul8(i: u8, j: u8) -> u8 {
    ((i as u16 * j as u16) & 0xFF) as u8
}

/// 8x8-bit multiplication, saturating at `0xFF`.
#[inline(always)]
pub const fn qmul8(i: u8, j: u8) -> u8 {
    let p = i as u16 * j as u16;
    if p > 255 { 255 } else { p as u8 }
}

/// Absolute value of a signed byte.
///
/// Note: like the original C version (and like [`i8::wrapping_abs`]), the
/// result of `abs8(i8::MIN)` is `i8::MIN`, since `-(-128)` does not fit in an
/// `i8`.
#[inline(always)]
pub const fn abs8(i: i8) -> i8 {
    i.wrapping_abs()
}

/// Blend `a` towards `b` by `amount_of_b / 256`, with 8-bit precision.
///
/// `amount_of_b == 0` returns `a` unchanged; `amount_of_b == 255` returns
/// (very nearly) `b`. Uses the rounding formula
/// `((a << 8) + (b - a) * amount_of_b + 0x80) >> 8`.
#[inline(always)]
pub const fn blend8_8bit(a: u8, b: u8, amount_of_b: u8) -> u8 {
    let mut partial: u16 = (a as u16) << 8;
    partial = partial.wrapping_add(b as u16 * amount_of_b as u16);
    partial = partial.wrapping_sub(a as u16 * amount_of_b as u16);
    partial = partial.wrapping_add(0x80);
    (partial >> 8) as u8
}

/// Blend `a` towards `b` by `amount_of_b / 256`, with 8-bit precision and
/// full range: `amount_of_b == 255` returns `b` *exactly*, mirroring
/// [`scale8`](crate::scale8)'s fixed-point convention.
///
/// The difference from [`blend8_8bit`] is the bias term. Both compute
/// `(a * 256 + (b - a) * amount_of_b + bias) >> 8`; [`blend8_8bit`] uses
/// `bias = 0x80` to round to nearest, which tops out one short of `b`
/// (`blend8_8bit(0, 255, 255) == 254`), while this uses `bias = b`, which
/// lands the top of the range exactly (`255`) at the cost of a slight
/// upward skew elsewhere.
///
/// This is FastLED's own `BLEND_FIXED + SCALE8_FIXED` formula, and was the
/// behavior of its `blend8` through the 3.6.x line; upstream later switched
/// the default to the rounding variant. Ported so that code targeting the
/// earlier behavior can reproduce it bit-for-bit.
#[inline(always)]
pub const fn blend8_8bit_full_range(a: u8, b: u8, amount_of_b: u8) -> u8 {
    // partial = a * 256 + b
    let mut partial: u16 = ((a as u16) << 8) | b as u16;
    partial = partial.wrapping_add(b as u16 * amount_of_b as u16);
    partial = partial.wrapping_sub(a as u16 * amount_of_b as u16);
    (partial >> 8) as u8
}

/// Blend `a` towards `b` by `amount_of_b / 256`, with 16-bit intermediate
/// precision for a more accurate result than [`blend8_8bit`].
#[inline(always)]
pub const fn blend8_16bit(a: u8, b: u8, amount_of_b: u8) -> u8 {
    let delta = b as i16 - a as i16;
    let mut partial: u32 = (a as u32) << 16;
    partial = partial.wrapping_add((delta as i32 * amount_of_b as i32 * 257) as u32);
    partial = partial.wrapping_add(0x8000);
    (partial >> 16) as u8
}

/// Blend `a` towards `b` by `amount_of_b / 256`.
///
/// Equivalent to [`blend8_16bit`] (FastLED selects between the 8-bit and
/// 16-bit precision variants based on available memory; on any target this
/// crate supports, the higher-precision version is used).
#[inline(always)]
pub const fn blend8(a: u8, b: u8, amount_of_b: u8) -> u8 {
    blend8_16bit(a, b, amount_of_b)
}

/// Remainder of `a / m`, i.e. `a % m`.
///
/// # Panics
/// Panics if `m == 0`, exactly like the built-in `%` operator it is
/// implemented in terms of (the original C version instead loops forever).
#[inline(always)]
pub const fn mod8(a: u8, m: u8) -> u8 {
    a % m
}

/// `(a + b) % m`, for incrementing a "mode" counter that wraps at `m`.
///
/// # Panics
/// Panics if `m == 0`.
#[inline(always)]
pub const fn addmod8(a: u8, b: u8, m: u8) -> u8 {
    a.wrapping_add(b) % m
}

/// `(a - b) % m`, for decrementing a "mode" counter that wraps at `m`.
///
/// # Panics
/// Panics if `m == 0`.
#[inline(always)]
pub const fn submod8(a: u8, b: u8, m: u8) -> u8 {
    a.wrapping_sub(b) % m
}

/// Integer square root of a 16-bit value, returned as a byte (i.e. clamped to
/// `0..=255`, which is exact for any input where the true root fits in a byte).
pub const fn sqrt16(x: u16) -> u8 {
    if x <= 1 {
        return x as u8;
    }

    let mut low: u8 = 1;
    let mut hi: u8 = if x > 7904 { 255 } else { ((x >> 5) + 8) as u8 };

    loop {
        let mid: u8 = ((low as u16 + hi as u16) >> 1) as u8;
        if (mid as u16 * mid as u16) > x {
            hi = mid - 1;
        } else {
            if mid == 255 {
                return 255;
            }
            low = mid + 1;
        }
        if hi < low {
            break;
        }
    }

    low - 1
}

/// Integer square root of a byte, returned as a byte.
///
/// Implemented by scaling `x` up to 16 bits (preserving its relative position
/// in the range) and delegating to [`sqrt16`], exactly as FastLED does.
#[inline(always)]
pub fn sqrt8(x: u8) -> u8 {
    sqrt16(int_scale::<u8, u16>(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturating_add_sub() {
        assert_eq!(qadd8(200, 100), 255);
        assert_eq!(qadd8(10, 20), 30);
        assert_eq!(qsub8(10, 20), 0);
        assert_eq!(qsub8(30, 10), 20);
        assert_eq!(qadd7(100, 100), 127);
        assert_eq!(qadd7(-100, -100), -128);
    }

    #[test]
    fn wrapping_add_sub() {
        assert_eq!(add8(200, 100), 44);
        assert_eq!(sub8(10, 20), 246);
        assert_eq!(add8to16(200, 65500), 164); // (200 + 65500) mod 65536
    }

    #[test]
    fn averages() {
        assert_eq!(avg8(10, 11), 10);
        assert_eq!(avg8r(10, 11), 11);
        assert_eq!(avg16(10, 11), 10);
        assert_eq!(avg16r(10, 11), 11);
        assert_eq!(avg7(4, 6), 5);
        assert_eq!(avg15(4, 6), 5);
    }

    #[test]
    fn multiplication() {
        assert_eq!(mul8(100, 100), (10000u32 & 0xFF) as u8);
        assert_eq!(qmul8(100, 100), 255);
        assert_eq!(qmul8(2, 3), 6);
    }

    #[test]
    fn absolute_value() {
        assert_eq!(abs8(-5), 5);
        assert_eq!(abs8(5), 5);
        assert_eq!(abs8(i8::MIN), i8::MIN);
    }

    #[test]
    fn blending() {
        assert_eq!(blend8(100, 200, 0), 100);
        assert_eq!(blend8(100, 200, 255), 200);
        assert_eq!(blend8_8bit(0, 255, 128), 128);
        assert_eq!(blend8_16bit(0, 255, 128), 128);
    }

    #[test]
    fn modulo() {
        assert_eq!(mod8(10, 7), 3);
        assert_eq!(addmod8(5, 3, 7), 1);
        // a -= b wraps in u8 first (2 - 5 -> 253), *then* reduces mod m: 253 % 7 == 1.
        assert_eq!(submod8(2, 5, 7), 1);
    }

    #[test]
    #[should_panic]
    fn mod8_by_zero_panics() {
        let _ = mod8(10, 0);
    }

    #[test]
    fn integer_sqrt() {
        assert_eq!(sqrt16(0), 0);
        assert_eq!(sqrt16(1), 1);
        assert_eq!(sqrt16(16), 4);
        assert_eq!(sqrt16(65535), 255);
        assert_eq!(sqrt8(255), 255);
        assert_eq!(sqrt8(0), 0);
    }
}
