//! Fast 8-/16-/32-bit scaling, video-safe scaling, and gamma-ish dimming /
//! brightening curves.
//!
//! "Scaling" a byte `i` by a [`Fract8`](crate::Fract8) `scale` computes
//! `i * (scale + 1) / 256` — i.e. `scale` is treated as a fixed-point
//! fraction in `0/256 ..= 256/256`, so `scale8(x, 255) == x` (full-range
//! round-trips exactly). This matches FastLED's default
//! `FASTLED_SCALE8_FIXED == 1` behavior.
//!
//! These are direct ports of the portable-C implementations in FastLED's
//! `scale8.h` (the `*_LEAVING_R1_DIRTY` / `cleanup_R1` family exists purely
//! to manage an AVR assembly register and has no portable meaning, so it is
//! not ported).

use crate::Fract8;
use crate::Fract16;

/// Scale a byte by a fixed-point fraction `scale / 256`.
///
/// `scale8(x, 0) == 0` and `scale8(x, 255) == x` (the maximum scale factor
/// maps back to the input exactly).
#[inline(always)]
pub const fn scale8(i: u8, scale: Fract8) -> u8 {
    (((i as u16) * (1 + scale as u16)) >> 8) as u8
}

/// `const`-evaluable version of [`scale8`], for use in `const` contexts.
#[inline(always)]
pub const fn scale8_constexpr(i: u8, scale: Fract8) -> u8 {
    scale8(i, scale)
}

/// Scale a byte by a fixed-point fraction `scale / 256`, "video" style: a
/// non-zero input always maps to a non-zero output, no matter how small
/// `scale` is (as long as it's non-zero). Useful for dimming LEDs without
/// ever fully extinguishing a lit pixel.
#[inline(always)]
pub const fn scale8_video(i: u8, scale: Fract8) -> u8 {
    let j = ((i as i32 * scale as i32) >> 8) as u8;
    j + (((i != 0) && (scale != 0)) as u8)
}

/// Scale `r`, `g` and `b` in place by a common fixed-point fraction
/// `scale / 256`. Equivalent to calling [`scale8`] on each component.
#[inline(always)]
pub fn nscale8x3(r: &mut u8, g: &mut u8, b: &mut u8, scale: Fract8) {
    *r = scale8(*r, scale);
    *g = scale8(*g, scale);
    *b = scale8(*b, scale);
}

/// `const`-evaluable RGB scaling: returns the scaled triple instead of
/// mutating in place, so it can be used to build `const` color tables.
///
/// Note: unlike [`scale8_constexpr`], this mirrors FastLED's
/// `nscale8x3_constexpr` exactly, which (likely a long-standing quirk in the
/// original) computes `r * scale / 256` *without* the `+1` fixed-point
/// rounding adjustment that [`scale8`]/[`nscale8x3`] use. So
/// `nscale8x3_constexpr(x, x, x, 255) != (x, x, x)` in general — use
/// [`nscale8x3`] if you need the round-trip-at-max property.
#[inline(always)]
pub const fn nscale8x3_constexpr(r: u8, g: u8, b: u8, scale: Fract8) -> (u8, u8, u8) {
    (
        ((r as u32 * scale as u32) >> 8) as u8,
        ((g as u32 * scale as u32) >> 8) as u8,
        ((b as u32 * scale as u32) >> 8) as u8,
    )
}

/// Scale `r`, `g` and `b` in place by a common fixed-point fraction
/// `scale / 256`, "video" style — see [`scale8_video`].
#[inline(always)]
pub fn nscale8x3_video(r: &mut u8, g: &mut u8, b: &mut u8, scale: Fract8) {
    *r = scale8_video(*r, scale);
    *g = scale8_video(*g, scale);
    *b = scale8_video(*b, scale);
}

/// Scale `i` and `j` in place by a common fixed-point fraction `scale / 256`.
#[inline(always)]
pub fn nscale8x2(i: &mut u8, j: &mut u8, scale: Fract8) {
    *i = scale8(*i, scale);
    *j = scale8(*j, scale);
}

/// Scale `i` and `j` in place by a common fixed-point fraction `scale / 256`,
/// "video" style — see [`scale8_video`].
#[inline(always)]
pub fn nscale8x2_video(i: &mut u8, j: &mut u8, scale: Fract8) {
    *i = scale8_video(*i, scale);
    *j = scale8_video(*j, scale);
}

/// Scale every byte in a slice in place by a common fixed-point fraction
/// `scale / 256` — equivalent to calling [`scale8`] on each element.
///
/// Generalizes [`nscale8x2`]/[`nscale8x3`] to arbitrary-length buffers; the
/// natural way to dim an entire LED strip (or any other byte buffer, e.g. a
/// flattened `[r, g, b, r, g, b, ...]` array) by a single common factor in
/// one pass. FastLED's analogous array-oriented helpers
/// (`nscale8`/`fadeToBlackBy`/etc. in `colorutils.h`) operate on its `CRGB`
/// color type; this crate has no color type of its own, so the natural
/// `no_std` shape is to operate directly on the underlying byte slice.
#[inline]
pub fn nscale8(values: &mut [u8], scale: Fract8) {
    for v in values {
        *v = scale8(*v, scale);
    }
}

/// Like [`nscale8`], but "video" style — see [`scale8_video`]: a non-zero
/// byte never scales down to zero, no matter how small `scale` is (as long
/// as it's non-zero). Useful for dimming a whole strip without ever fully
/// extinguishing lit pixels.
#[inline]
pub fn nscale8_video(values: &mut [u8], scale: Fract8) {
    for v in values {
        *v = scale8_video(*v, scale);
    }
}

/// Scale a 16-bit value by an 8-bit fixed-point fraction `scale / 256`.
#[inline(always)]
pub const fn scale16by8(i: u16, scale: Fract8) -> u16 {
    if scale == 0 {
        0
    } else {
        (((i as u32) * (1 + scale as u32)) >> 8) as u16
    }
}

/// Scale a 16-bit value by a 16-bit fixed-point fraction `scale / 65536`.
#[inline(always)]
pub const fn scale16(i: u16, scale: Fract16) -> u16 {
    (((i as u32) * (1 + scale as u32)) / 65536) as u16
}

/// Scale a 32-bit value by an 8-bit fixed-point fraction `scale / 256`.
/// Uses a 64-bit intermediate to avoid overflow.
#[inline(always)]
pub const fn scale32by8(i: u32, scale: Fract8) -> u32 {
    if scale == 0 {
        0
    } else {
        (((i as u64) * (1 + scale as u64)) >> 8) as u32
    }
}

/// Apply a gamma-2-ish dimming curve: `scale8(x, x)`.
///
/// The eye perceives brightness non-linearly, so a linear PWM duty cycle of
/// 50% looks much brighter than "half as bright". This (and its siblings)
/// approximate gamma correction with `gamma ≈ 2.0` so that a midpoint value
/// (128) *looks* like it's about half as bright as full (255).
#[inline(always)]
pub const fn dim8_raw(x: u8) -> u8 {
    scale8(x, x)
}

/// Like [`dim8_raw`], but "video" style — the result never drops to zero for
/// a non-zero input.
#[inline(always)]
pub const fn dim8_video(x: u8) -> u8 {
    scale8_video(x, x)
}

/// Linear-ish dimming curve: halves values below the midpoint, applies
/// [`dim8_raw`]'s curve above it. A gentler dimming curve than [`dim8_raw`].
#[inline(always)]
pub const fn dim8_lin(x: u8) -> u8 {
    if x & 0x80 != 0 {
        scale8(x, x)
    } else {
        (x + 1) / 2
    }
}

/// Inverse of [`dim8_raw`]: brighten a dimmed value back towards full scale.
#[inline(always)]
pub const fn brighten8_raw(x: u8) -> u8 {
    let ix = 255 - x;
    255 - scale8(ix, ix)
}

/// Inverse of [`dim8_video`].
#[inline(always)]
pub const fn brighten8_video(x: u8) -> u8 {
    let ix = 255 - x;
    255 - scale8_video(ix, ix)
}

/// Inverse of [`dim8_lin`].
#[inline(always)]
pub const fn brighten8_lin(x: u8) -> u8 {
    let ix = 255 - x;
    let out = if ix & 0x80 != 0 {
        scale8(ix, ix)
    } else {
        (ix + 1) / 2
    };
    255 - out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale8_round_trips_at_max() {
        for x in 0..=255u8 {
            assert_eq!(scale8(x, 255), x);
            assert_eq!(scale8(x, 0), 0);
        }
    }

    #[test]
    fn scale8_known_values() {
        assert_eq!(scale8(255, 128), 128);
        assert_eq!(scale8(128, 128), 64);
        assert_eq!(scale8_constexpr(255, 128), scale8(255, 128));
    }

    #[test]
    fn scale8_video_preserves_nonzero() {
        for scale in 1..=255u8 {
            assert_ne!(scale8_video(1, scale), 0, "scale={scale}");
        }
        assert_eq!(scale8_video(0, 255), 0);
        assert_eq!(scale8_video(10, 0), 0);
        assert_eq!(scale8_video(255, 255), 255);
    }

    #[test]
    fn nscale_in_place_matches_scale8() {
        let (mut r, mut g, mut b) = (200u8, 100u8, 50u8);
        nscale8x3(&mut r, &mut g, &mut b, 128);
        assert_eq!(
            (r, g, b),
            (scale8(200, 128), scale8(100, 128), scale8(50, 128))
        );

        let (mut i, mut j) = (200u8, 50u8);
        nscale8x2(&mut i, &mut j, 64);
        assert_eq!((i, j), (scale8(200, 64), scale8(50, 64)));
    }

    #[test]
    fn nscale8_dims_a_whole_slice_in_place() {
        let mut strip = [200u8, 100, 50, 255, 0];
        let expected: [u8; 5] = strip.map(|x| scale8(x, 64));
        nscale8(&mut strip, 64);
        assert_eq!(strip, expected);

        // scale == 0 turns everything off; scale == 255 round-trips exactly.
        let mut strip = [10u8, 20, 30];
        nscale8(&mut strip, 0);
        assert_eq!(strip, [0, 0, 0]);

        let mut strip = [10u8, 20, 30];
        nscale8(&mut strip, 255);
        assert_eq!(strip, [10, 20, 30]);
    }

    #[test]
    fn nscale8_video_never_zeroes_a_lit_pixel() {
        let mut strip = [1u8, 50, 255];
        nscale8_video(&mut strip, 1);
        assert!(strip.iter().all(|&x| x != 0), "{strip:?}");

        let mut strip = [1u8, 50, 255];
        nscale8_video(&mut strip, 0);
        assert_eq!(strip, [0, 0, 0]);
    }

    #[test]
    fn wide_scaling() {
        assert_eq!(scale16by8(0, 200), 0);
        assert_eq!(scale16by8(65535, 0), 0);
        assert_eq!(scale16by8(65535, 255), 65535);
        assert_eq!(scale16(65535, 65535), 65535);
        assert_eq!(scale32by8(u32::MAX, 255), u32::MAX);
        assert_eq!(scale32by8(u32::MAX, 0), 0);
    }

    #[test]
    fn dimming_and_brightening_are_inverses_at_extremes() {
        assert_eq!(dim8_raw(0), 0);
        assert_eq!(dim8_raw(255), 255);
        assert_eq!(brighten8_raw(0), 0);
        assert_eq!(brighten8_raw(255), 255);
        assert_eq!(dim8_lin(0), 0);
        assert_eq!(dim8_lin(255), 255);
        assert_eq!(brighten8_lin(0), 0);
        assert_eq!(brighten8_lin(255), 255);
    }
}
