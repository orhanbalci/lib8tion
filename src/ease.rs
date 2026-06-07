//! Easing curves and waveform generators — turning a linearly increasing
//! counter into a value that accelerates and decelerates smoothly, the way a
//! physical object would when starting and stopping.
//!
//! Direct port of the `Easing` and `WaveformGenerators` groups in FastLED's
//! `lib8tion.h`. C names are `camelCase` (`ease8InOutCubic`); this port uses
//! `snake_case` (`ease8_in_out_cubic`) per Rust convention, but is otherwise a
//! line-for-line translation — `scale8_LEAVING_R1_DIRTY` is FastLED's
//! AVR-asm-register-cleanup hint for [`scale8`](crate::scale8::scale8) and is
//! ported as a plain call to it (see also the note on `nscale8x3_constexpr`
//! in [`scale8`](crate::scale8) — these AVR-only naming quirks have no
//! portable semantic meaning).
//!
//! See <https://easings.net> for a visual reference of these curve shapes.

use crate::scale8::{scale8, scale16};
use crate::{Fract8, Fract16};

/// 8-bit quadratic ease-in/ease-out: an S-curve that starts and ends slowly
/// and moves fastest through the middle, built from `scale8(x, x)` (i.e.
/// `x^2`) mirrored across the midpoint.
#[inline]
pub const fn ease8_in_out_quad(i: u8) -> u8 {
    let mut j = i;
    if j & 0x80 != 0 {
        j = 255 - j;
    }
    let jj = scale8(j, Fract8(j));
    let mut jj2 = jj << 1;
    if i & 0x80 != 0 {
        jj2 = 255 - jj2;
    }
    jj2
}

/// 16-bit quadratic ease-in/ease-out — see [`ease8_in_out_quad`].
#[inline]
pub const fn ease16_in_out_quad(i: u16) -> u16 {
    let mut j = i;
    if j & 0x8000 != 0 {
        j = 65535 - j;
    }
    let jj = scale16(j, Fract16(j));
    let mut jj2 = jj << 1;
    if i & 0x8000 != 0 {
        jj2 = 65535 - jj2;
    }
    jj2
}

/// 8-bit cubic ease-in/ease-out: a steeper S-curve than [`ease8_in_out_quad`]
/// that spends visibly more time at the extremes, computed directly from the
/// textbook smoothstep polynomial `3x^2 - 2x^3` (in `0..=1` fixed-point
/// terms).
#[inline]
pub const fn ease8_in_out_cubic(i: u8) -> u8 {
    let ii = scale8(i, Fract8(i));
    let iii = scale8(ii, Fract8(i));

    let r1 = (3 * ii as u16).wrapping_sub(2 * iii as u16);

    // Rounding/quantization error can produce "256"; clamp it to 255.
    if r1 & 0x100 != 0 { 255 } else { r1 as u8 }
}

/// 16-bit cubic ease-in/ease-out — see [`ease8_in_out_cubic`].
///
/// Note: FastLED's own header remarks that this legacy 16-bit version
/// "produces wrong results" relative to its newer floating-point-accurate
/// replacement (`fl::easeInOutCubic16`, not part of portable `lib8tion`) —
/// it's reproduced here verbatim (quirks included) for parity with the
/// `LIB8STATIC` surface this crate ports.
#[inline]
pub const fn ease16_in_out_cubic(i: u16) -> u16 {
    let ii = scale16(i, Fract16(i)) as u32;
    let iii = scale16(ii as u16, Fract16(i)) as u32;

    let r1 = (3 * ii).wrapping_sub(2 * iii);

    if r1 > 65535 { 65535 } else { r1 as u16 }
}

/// Fast, rough approximation of [`ease8_in_out_cubic`]'s S-curve, built from
/// three linear segments (slopes 0.5, 1.5, 0.5) instead of the cubic
/// polynomial. Never off by more than a couple of percent from the true
/// cubic curve, and noticeably cheaper to compute — use this when raw speed
/// matters more than curve precision.
#[inline]
pub const fn ease8_in_out_approx(i: u8) -> u8 {
    if i < 64 {
        // start with slope 0.5
        i / 2
    } else if i > 255 - 64 {
        // end with slope 0.5
        255 - ((255 - i) / 2)
    } else {
        // in the middle, use slope 1.5
        let mut j = i - 64;
        j += j / 2;
        j + 32
    }
}

/// Triangle wave generator: turns a linearly increasing counter into a value
/// that ramps up then back down (`0..=127` maps to a rising `0..=254`,
/// `128..=255` maps to a falling `254..=0`).
#[inline]
pub const fn triwave8(i: u8) -> u8 {
    let j = if i & 0x80 != 0 { 255 - i } else { i };
    j << 1
}

/// Quadratic ("sine-like") waveform generator: [`triwave8`] reshaped through
/// [`ease8_in_out_quad`] for a smoother, more "S-curved" oscillation —
/// cheaper to compute than [`crate::trig8::sin8`], with a slightly different
/// curve shape.
#[inline]
pub const fn quadwave8(i: u8) -> u8 {
    ease8_in_out_quad(triwave8(i))
}

/// Cubic ("sine-like") waveform generator — see [`quadwave8`], but spends
/// visibly more time near its extremes (a steeper S-curve, from
/// [`ease8_in_out_cubic`]).
#[inline]
pub const fn cubicwave8(i: u8) -> u8 {
    ease8_in_out_cubic(triwave8(i))
}

/// Square wave generator: outputs `255` while `i < pulse_width` and `0`
/// otherwise (with `pulse_width == 255` always producing `255`, so the pulse
/// can be made permanently "on"). Useful for blinking/strobing effects, or as
/// a building block for pulse-width-modulated patterns.
#[inline]
pub const fn squarewave8(i: u8, pulse_width: u8) -> u8 {
    if i < pulse_width || pulse_width == 255 {
        255
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease8_in_out_quad_is_a_symmetric_s_curve() {
        assert_eq!(ease8_in_out_quad(0), 0);
        assert_eq!(ease8_in_out_quad(255), 255);
        assert_eq!(ease8_in_out_quad(128), 129);
        // Symmetric about the midpoint: easing in mirrors easing out.
        for i in 0..=127u8 {
            assert_eq!(
                ease8_in_out_quad(i),
                255 - ease8_in_out_quad(255 - i),
                "i={i}"
            );
        }
    }

    #[test]
    fn ease16_in_out_quad_is_a_symmetric_s_curve() {
        assert_eq!(ease16_in_out_quad(0), 0);
        assert_eq!(ease16_in_out_quad(65535), 65535);
        for i in (0..=32767u16).step_by(733) {
            assert_eq!(
                ease16_in_out_quad(i),
                65535 - ease16_in_out_quad(65535 - i),
                "i={i}"
            );
        }
    }

    #[test]
    fn ease8_in_out_cubic_anchors_and_midpoint() {
        assert_eq!(ease8_in_out_cubic(0), 0);
        assert_eq!(ease8_in_out_cubic(255), 255);
        // 3(0.5)^2 - 2(0.5)^3 == 0.5: the curve passes through its midpoint.
        assert_eq!(ease8_in_out_cubic(128), 128);
    }

    #[test]
    fn ease16_in_out_cubic_anchors_and_midpoint() {
        assert_eq!(ease16_in_out_cubic(0), 0);
        assert_eq!(ease16_in_out_cubic(65535), 65535);
    }

    #[test]
    fn ease8_in_out_approx_anchors_and_midpoint() {
        assert_eq!(ease8_in_out_approx(0), 0);
        assert_eq!(ease8_in_out_approx(255), 255);
        assert_eq!(ease8_in_out_approx(128), 128);
        // Stays close to the true cubic curve everywhere (never off by more
        // than a couple of percent, per FastLED's docs for this function).
        for i in 0..=255u8 {
            let exact = ease8_in_out_cubic(i) as i16;
            let approx = ease8_in_out_approx(i) as i16;
            assert!(
                (exact - approx).abs() <= 8,
                "i={i} exact={exact} approx={approx}"
            );
        }
    }

    #[test]
    fn triwave8_ramps_up_then_down() {
        assert_eq!(triwave8(0), 0);
        assert_eq!(triwave8(64), 128);
        assert_eq!(triwave8(127), 254);
        assert_eq!(triwave8(128), 254);
        assert_eq!(triwave8(192), 126);
        assert_eq!(triwave8(255), 0);
    }

    #[test]
    fn quad_and_cubic_waves_are_bounded_and_periodic() {
        for i in 0..=255u8 {
            let _ = quadwave8(i);
            let _ = cubicwave8(i);
        }
        // Both waves are built from triwave8, which is symmetric about 128.
        assert_eq!(quadwave8(0), quadwave8(255));
        assert_eq!(cubicwave8(0), cubicwave8(255));
    }

    #[test]
    fn squarewave8_switches_at_pulse_width() {
        for i in 0..100u8 {
            assert_eq!(squarewave8(i, 100), 255);
        }
        for i in 100..=255u8 {
            assert_eq!(squarewave8(i, 100), 0);
        }
        // pulse_width == 255 means "always on".
        for i in 0..=255u8 {
            assert_eq!(squarewave8(i, 255), 255);
        }
    }
}
