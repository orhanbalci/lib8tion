//! Fast linear interpolation ("lerp") between two integer values, the
//! building block for things like Perlin noise, color blending, and smooth
//! parameter sweeps. [`crate::math8::blend8`] is essentially `lerp8by8` for
//! color channels; these are the general-purpose primitives it (and you) can
//! build on.
//!
//! Direct port of the `LinearInterpolation` group in FastLED's `lib8tion.h`.
//!
//! All of these compute `a + (b - a) * frac` (i.e. `frac == 0` returns `a`,
//! `frac == max` returns (very nearly) `b`), but — per FastLED's own
//! comment — split the `b > a` / `b <= a` cases so the `b - a` subtraction
//! never has to be promoted to a wider type to avoid overflow: each case
//! computes a non-negative `delta` in the input width, scales it by `frac`,
//! and adds or subtracts it from `a` accordingly.

use crate::scale8::{scale8, scale16, scale16by8};

/// Linearly interpolate between two unsigned bytes, with an 8-bit fraction.
#[inline]
pub const fn lerp8by8(a: u8, b: u8, frac: u8) -> u8 {
    if b > a {
        let delta = b - a;
        let scaled = scale8(delta, frac);
        a + scaled
    } else {
        let delta = a - b;
        let scaled = scale8(delta, frac);
        a - scaled
    }
}

/// Linearly interpolate between two unsigned 16-bit values, with a 16-bit
/// fraction.
#[inline]
pub const fn lerp16by16(a: u16, b: u16, frac: u16) -> u16 {
    if b > a {
        let delta = b - a;
        let scaled = scale16(delta, frac);
        a + scaled
    } else {
        let delta = a - b;
        let scaled = scale16(delta, frac);
        a - scaled
    }
}

/// Linearly interpolate between two unsigned 16-bit values, with an 8-bit
/// fraction.
#[inline]
pub const fn lerp16by8(a: u16, b: u16, frac: u8) -> u16 {
    if b > a {
        let delta = b - a;
        let scaled = scale16by8(delta, frac);
        a + scaled
    } else {
        let delta = a - b;
        let scaled = scale16by8(delta, frac);
        a - scaled
    }
}

/// Linearly interpolate between two signed 15-bit values (held in `i16`),
/// with an 8-bit fraction.
///
/// The intermediate `delta`/`scaled` values are computed in `i32` (matching
/// C's `int`-promotion of the `i16` operands) and only truncated back to
/// `i16` at the end — exactly mirroring FastLED's behavior, including its
/// wrap-on-truncation for any out-of-range inputs.
#[inline]
pub const fn lerp15by8(a: i16, b: i16, frac: u8) -> i16 {
    if b > a {
        let delta = (b as i32 - a as i32) as u16;
        let scaled = scale16by8(delta, frac);
        (a as i32 + scaled as i32) as i16
    } else {
        let delta = (a as i32 - b as i32) as u16;
        let scaled = scale16by8(delta, frac);
        (a as i32 - scaled as i32) as i16
    }
}

/// Linearly interpolate between two signed 15-bit values (held in `i16`),
/// with a 16-bit fraction. See [`lerp15by8`] for the intermediate-precision
/// note.
#[inline]
pub const fn lerp15by16(a: i16, b: i16, frac: u16) -> i16 {
    if b > a {
        let delta = (b as i32 - a as i32) as u16;
        let scaled = scale16(delta, frac);
        (a as i32 + scaled as i32) as i16
    } else {
        let delta = (a as i32 - b as i32) as u16;
        let scaled = scale16(delta, frac);
        (a as i32 - scaled as i32) as i16
    }
}

/// Maps an 8-bit input from its full range (`0..=255`) into the narrower
/// range `range_start..=range_end`.
///
/// Mathematically similar to [`lerp8by8`], but with `map`-style arguments
/// (à la Arduino's `map(in, 0, 255, range_start, range_end)`, but faster and
/// purpose-built for bytes). Combines nicely with the waveform generators —
/// e.g. `map8(sin8(x), HUE_BLUE, HUE_RED)` sweeps a hue back and forth.
///
/// Note: like the original, this assumes `range_end >= range_start`; if
/// `range_end < range_start` the `range_width` subtraction wraps (per `u8`
/// arithmetic), producing a result outside the apparent "range".
#[inline]
pub const fn map8(input: u8, range_start: u8, range_end: u8) -> u8 {
    let range_width = range_end.wrapping_sub(range_start);
    let out = scale8(input, range_width);
    out.wrapping_add(range_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp8by8_interpolates_between_endpoints() {
        assert_eq!(lerp8by8(0, 255, 0), 0);
        assert_eq!(lerp8by8(0, 255, 255), 255);
        assert_eq!(lerp8by8(0, 255, 128), 128);
        // Symmetric: interpolating from b to a by `frac` matches a to b by
        // `255 - frac` (within rounding).
        assert_eq!(lerp8by8(10, 200, 64), lerp8by8(200, 10, 255 - 64));
    }

    #[test]
    fn lerp16by16_interpolates_between_endpoints() {
        assert_eq!(lerp16by16(0, 65535, 0), 0);
        assert_eq!(lerp16by16(0, 65535, 65535), 65535);
        assert_eq!(lerp16by16(1000, 100, 65535), 100);
    }

    #[test]
    fn lerp16by8_interpolates_between_endpoints() {
        assert_eq!(lerp16by8(0, 65535, 0), 0);
        assert_eq!(lerp16by8(0, 65535, 255), 65535);
        assert_eq!(lerp16by8(1000, 100, 255), 100);
    }

    #[test]
    fn lerp15by8_handles_signed_endpoints() {
        assert_eq!(lerp15by8(-1000, 1000, 0), -1000);
        assert_eq!(lerp15by8(-1000, 1000, 255), 1000);
        assert_eq!(lerp15by8(1000, -1000, 255), -1000);
        // Halfway between symmetric endpoints lands at (about) zero:
        // delta=2000, scale16by8(2000, 128) == 1007, -1000 + 1007 == 7.
        assert_eq!(lerp15by8(-1000, 1000, 128), 7);
    }

    #[test]
    fn lerp15by16_handles_signed_endpoints() {
        assert_eq!(lerp15by16(-1000, 1000, 0), -1000);
        assert_eq!(lerp15by16(-1000, 1000, 65535), 1000);
        assert_eq!(lerp15by16(1000, -1000, 65535), -1000);
    }

    #[test]
    fn map8_remaps_full_range_into_a_narrower_one() {
        assert_eq!(map8(0, 100, 200), 100);
        // range_width=100; scale8(255, 100) == (255*101)>>8 == 100; +100 == 200.
        assert_eq!(map8(255, 100, 200), 200);
        assert_eq!(map8(128, 0, 255), 128);
        for input in 0..=255u8 {
            let out = map8(input, 100, 200);
            assert!((100..=200).contains(&out), "input={input} out={out}");
        }
    }
}
