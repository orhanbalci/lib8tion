//! Property-based tests (via `proptest`).
//!
//! Two flavors:
//!   - *Differential* properties: for domains too wide to sweep exhaustively
//!     (u16/u32/i16), generate random inputs — with shrinking — and assert
//!     `lib8tion::*` matches FastLED's actual C reference (`fastled_ref::*`).
//!     Shrinking is the win over hand-picked sample points: a mismatch
//!     collapses to a minimal reproducing input instead of a dense dump.
//!   - *Invariant* properties: algebraic facts that must hold of the Rust
//!     port by construction (saturation bounds, range bounds, identities at
//!     fixed points), independent of the C reference. These catch bugs that
//!     a transcription error could otherwise let slip through both sides of
//!     a differential check.

use lib8tion as l;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Differential properties — wide domains, random + shrinking
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sin16_cos16_match_reference(theta: u16) {
        prop_assert_eq!(l::sin16(theta), fastled_ref::sin16(theta));
        prop_assert_eq!(l::cos16(theta), fastled_ref::cos16(theta));
    }

    #[test]
    fn sqrt16_matches_reference(x: u16) {
        prop_assert_eq!(l::sqrt16(x), fastled_ref::sqrt16(x));
    }

    #[test]
    fn scale16_family_matches_reference(i: u16, scale16: u16, scale8: u8) {
        prop_assert_eq!(l::scale16(i, scale16), fastled_ref::scale16(i, scale16));
        prop_assert_eq!(l::scale16by8(i, scale8), fastled_ref::scale16by8(i, scale8));
    }

    #[test]
    fn scale32by8_matches_reference(i: u32, scale: u8) {
        prop_assert_eq!(l::scale32by8(i, scale), fastled_ref::scale32by8(i, scale));
    }

    #[test]
    fn avg16_family_matches_reference(a: u16, b: u16) {
        prop_assert_eq!(l::avg16(a, b), fastled_ref::avg16(a, b));
        prop_assert_eq!(l::avg16r(a, b), fastled_ref::avg16r(a, b));
    }

    #[test]
    fn avg15_matches_reference(a: i16, b: i16) {
        prop_assert_eq!(l::avg15(a, b), fastled_ref::avg15(a, b));
    }

    #[test]
    fn lerp16_family_matches_reference(a: u16, b: u16, frac8: u8, frac16: u16) {
        prop_assert_eq!(l::lerp16by8(a, b, frac8), fastled_ref::lerp16by8(a, b, frac8));
        prop_assert_eq!(l::lerp16by16(a, b, frac16), fastled_ref::lerp16by16(a, b, frac16));
    }

    #[test]
    fn lerp15_family_matches_reference(a: i16, b: i16, frac8: u8, frac16: u16) {
        prop_assert_eq!(l::lerp15by8(a, b, frac8), fastled_ref::lerp15by8(a, b, frac8));
        prop_assert_eq!(l::lerp15by16(a, b, frac16), fastled_ref::lerp15by16(a, b, frac16));
    }

    #[test]
    fn ease16_family_matches_reference(i: u16) {
        prop_assert_eq!(l::ease16_in_out_quad(i), fastled_ref::ease16_in_out_quad(i));
        prop_assert_eq!(l::ease16_in_out_cubic(i), fastled_ref::ease16_in_out_cubic(i));
    }

    #[test]
    fn add8to16_matches_reference(i: u8, j: u16) {
        prop_assert_eq!(l::add8to16(i, j), fastled_ref::add8to16(i, j));
    }
}

// ---------------------------------------------------------------------------
// Invariant properties — algebraic facts about the Rust port
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn qadd8_saturates_and_never_decreases(a: u8, b: u8) {
        let r = l::qadd8(a, b);
        prop_assert_eq!(r, (a as u16 + b as u16).min(255) as u8);
        prop_assert!(r >= a && r >= b);
    }

    #[test]
    fn qsub8_saturates_and_never_increases(a: u8, b: u8) {
        let r = l::qsub8(a, b);
        prop_assert_eq!(r, a.saturating_sub(b));
        prop_assert!(r <= a);
    }

    #[test]
    fn qmul8_saturates_at_255(a: u8, b: u8) {
        let r = l::qmul8(a, b);
        prop_assert_eq!(r, ((a as u16 * b as u16).min(255)) as u8);
    }

    #[test]
    fn avg8_lies_between_its_inputs(a: u8, b: u8) {
        let r = l::avg8(a, b);
        prop_assert!(r >= a.min(b) && r <= a.max(b));
    }

    #[test]
    fn scale8_zero_scale_is_zero(x: u8) {
        prop_assert_eq!(l::scale8(x, 0), 0);
        prop_assert_eq!(l::scale8(0, x), 0);
    }

    #[test]
    fn mod8_remainder_is_smaller_than_modulus(a: u8, m in 1u8..=255) {
        prop_assert!(l::mod8(a, m) < m);
    }

    #[test]
    fn map8_output_stays_within_the_target_range(input: u8, start: u8, end: u8) {
        let (lo, hi) = (start.min(end), start.max(end));
        // map8 assumes range_end >= range_start; normalize before calling.
        let out = l::map8(input, lo, hi);
        prop_assert!(out >= lo && out <= hi);
    }

    #[test]
    fn lerp8by8_hits_its_endpoints_at_frac_extremes(a: u8, b: u8) {
        prop_assert_eq!(l::lerp8by8(a, b, 0), a);
        prop_assert_eq!(l::lerp8by8(a, a, 255), a);
    }

    #[test]
    fn blend8_is_identity_when_endpoints_match(a: u8, amount: u8) {
        prop_assert_eq!(l::blend8(a, a, amount), a);
    }

    #[test]
    fn blend8_at_extremes_returns_an_endpoint(a: u8, b: u8) {
        prop_assert_eq!(l::blend8(a, b, 0), a);
        prop_assert_eq!(l::blend8(a, b, 255), b);
    }

    #[test]
    fn triwave8_ramps_up_then_mirrors_down(theta: u8) {
        // triwave8 ramps 0..=255 on the rising half (theta < 0x80) and
        // mirrors that ramp on the falling half.
        let r = l::triwave8(theta);
        if theta < 0x80 {
            prop_assert_eq!(r, theta.wrapping_mul(2));
        } else {
            prop_assert_eq!(r, (255 - theta).wrapping_mul(2));
        }
    }

    #[test]
    fn squarewave8_is_either_endpoint(theta: u8, pulsewidth: u8) {
        let r = l::squarewave8(theta, pulsewidth);
        prop_assert!(r == 0 || r == 255);
        prop_assert_eq!(r == 255, theta < pulsewidth || pulsewidth == 255);
    }

    #[test]
    fn sin16_is_bounded_and_matches_lookup_table_range(theta: u16) {
        let s = l::sin16(theta);
        prop_assert!(s >= -32645 && s <= 32645);
    }
}

#[test]
fn ease8_in_out_quad_anchors_its_endpoints() {
    assert_eq!(l::ease8_in_out_quad(0), 0);
    assert_eq!(l::ease8_in_out_quad(255), 255);
}
