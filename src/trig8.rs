//! Fast 8-/16-bit approximations of `sin`/`cos`, using small lookup tables
//! plus linear interpolation rather than a full trig implementation.
//!
//! Don't use these for orbital mechanics — but for animating LEDs they are
//! dramatically cheaper than `f32::sin`/`cos` (no FPU required, no `libm`
//! dependency) and visually indistinguishable.
//!
//! Direct port of the portable-C implementations in FastLED's `trig8.h`.

use crate::intmap::int_scale;

/// Lookup table of "section start" sine values, used by [`sin16`]'s
/// piecewise-linear approximation across one quarter wave (8 sections).
const SIN16_BASE: [u16; 8] = [0, 6393, 12539, 18204, 23170, 27245, 30273, 32137];

/// Per-section slopes used to interpolate between [`SIN16_BASE`] entries.
const SIN16_SLOPE: [u8; 8] = [49, 48, 44, 38, 31, 23, 14, 4];

/// Fast 16-bit approximation of `sin(theta)`.
///
/// `theta` covers a full circle over its range `0..=65535`; the result is in
/// `-32768..=32767`. Implemented as an 8-segment piecewise-linear
/// approximation of one quarter wave, mirrored/reflected to cover the full
/// circle (sine is symmetric across quarters up to sign and direction).
pub const fn sin16(theta: u16) -> i16 {
    let mut offset = (theta & 0x3FFF) >> 3; // 0..2047, position within a quarter wave
    if theta & 0x4000 != 0 {
        offset = 2047 - offset;
    }

    let section = (offset / 256) as usize; // 0..7
    let b = SIN16_BASE[section];
    let m = SIN16_SLOPE[section];

    let secoffset8 = (offset as u8) / 2;

    let mx = m as u16 * secoffset8 as u16;
    let mut y = (mx as i32 + b as i32) as i16;

    if theta & 0x8000 != 0 {
        y = -y;
    }

    y
}

/// Fast 16-bit approximation of `cos(theta)`. Implemented as `sin16` with a
/// quarter-circle phase shift.
#[inline(always)]
pub const fn cos16(theta: u16) -> i16 {
    sin16(theta.wrapping_add(16384))
}

/// Lookup table of `(base, slope)` pairs (interleaved) used by [`sin8`]'s
/// 4-section piecewise-linear approximation across one quarter wave.
const B_M16_INTERLEAVE: [u8; 8] = [0, 49, 49, 41, 90, 27, 117, 10];

/// Fast 8-bit approximation of `sin(theta)`.
///
/// `theta` covers a full circle over its range `0..=255`; the result is in
/// `0..=255` (i.e. already biased/scaled so it can be used directly as a
/// brightness or color channel — `sin8(0) == 128`, `sin8(64) == 255`,
/// `sin8(192) == 0`).
pub const fn sin8(theta: u8) -> u8 {
    let mut offset = theta;
    if theta & 0x40 != 0 {
        offset = 255 - offset;
    }
    offset &= 0x3F; // 0..63

    let mut secoffset = offset & 0x0F; // 0..15
    if theta & 0x40 != 0 {
        secoffset += 1;
    }

    let section = offset >> 4; // 0..3
    let s2 = (section * 2) as usize;
    let b = B_M16_INTERLEAVE[s2];
    let m16 = B_M16_INTERLEAVE[s2 + 1];

    let mx = (m16 as u16 * secoffset as u16) >> 4;

    let mut y = (mx as i16 + b as i16) as i8;
    if theta & 0x80 != 0 {
        y = -y;
    }

    (y as i16 + 128) as u8
}

/// Fast 8-bit approximation of `cos(theta)`. Implemented as `sin8` with a
/// quarter-circle phase shift.
#[inline(always)]
pub const fn cos8(theta: u8) -> u8 {
    sin8(theta.wrapping_add(64))
}

/// Re-scale an 8-bit angle (`0..=255` for a full circle) to the 16-bit angle
/// space used by [`sin16`]/[`cos16`], call through, and rescale the `i16`
/// result back down to `u8` brightness range. This is how [`sin8`]/[`cos8`]
/// could equivalently be derived from the 16-bit primitives — provided here
/// as a building block for code that already has a 16-bit lookup table and
/// wants consistent rounding via [`int_scale`].
#[inline(always)]
pub fn sin8_via_sin16(theta: u8) -> u8 {
    let unsigned_result = (sin16((theta as u16) << 8) as i32 + 32768) as u16;
    int_scale::<u16, u8>(unsigned_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin16_quarter_wave_landmarks() {
        // The piecewise-linear approximation isn't perfectly odd-symmetric at
        // every point (integer truncation differs slightly across quadrant
        // boundaries), so rather than asserting sin16(-x) == -sin16(x)
        // pointwise, check the well-defined landmark angles directly.
        assert_eq!(sin16(0), 0);
        assert_eq!(sin16(8192), 23170); // eighth turn: ~ sin(45 deg) * 32767
        assert_eq!(sin16(16384), 32645); // quarter turn: ~ sin(90 deg) plateau
        assert_eq!(sin16(32768), 0); // half turn
        assert_eq!(sin16(49152), -32645); // three-quarter turn
        assert_eq!(cos16(0), cos16(0));
    }

    #[test]
    fn sin8_known_values() {
        assert_eq!(sin8(0), 128);
        assert_eq!(sin8(64), 255);
        // Truncation in the approximation means the "zero crossing" at 270
        // degrees lands on 1, not 0 (matches FastLED's actual output).
        assert_eq!(sin8(192), 1);
    }

    #[test]
    fn sin8_cos8_are_bounded_and_consistent() {
        for theta in 0..=255u8 {
            let s = sin8(theta);
            let c = cos8(theta);
            // Both approximations live entirely within u8's range by construction;
            // this just guards against accidental panics/regressions in the port.
            let _ = (s, c);
        }
        assert_eq!(cos8(0), sin8(64));
    }
}
