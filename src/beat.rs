//! Beat generators — phase/value oscillators driven by a millisecond clock,
//! the building block FastLED animations use to "pulse" or "wave" values at a
//! given tempo (BPM).
//!
//! Direct port of the `beat8`/`beat16`/`beat88`/`beatsin8`/`beatsin16`/
//! `beatsin88` family in FastLED's `lib8tion.h`. The original reads the
//! current time from a global `millis()`; this port instead takes the
//! timestamp as an explicit `now_millis` argument — the same "no hidden
//! global state" shape as [`Rng16`](crate::Rng16), and the only sensible one
//! in `no_std` (there is no universal monotonic clock to reach for). Drive it
//! with whatever millisecond counter your platform provides (a `SysTick`,
//! an RTC, a simulation clock, ...).

use crate::scale8::{scale8, scale16};
use crate::trig8::{sin8, sin16};

/// Generates a 16-bit "beat" ramp from a Q8.8 fixed-point BPM value.
///
/// `bpm88` is beats-per-minute in `Q8.8` format (high byte = whole BPM, low
/// byte = fractional BPM in 1/256ths — e.g. `120 << 8` for "120.0 BPM", or
/// `(120 << 8) | 128` for "120.5 BPM"). `timebase` shifts the phase — the
/// ramp restarts when `now_millis == timebase`. Returns a value that ramps
/// from `0` to `65535` and wraps, completing one cycle per beat.
#[inline]
pub const fn beat88(bpm88: u16, timebase: u32, now_millis: u32) -> u16 {
    // 65536 ticks-per-beat-cycle : 60000 ms-per-minute reduces to ~280:256;
    // rounding to 280 keeps this a cheap multiply-and-shift (see FastLED's
    // comment in lib8tion.h) at the cost of beats running ~0.07% fast.
    let elapsed = now_millis.wrapping_sub(timebase);
    let scaled = elapsed.wrapping_mul(bpm88 as u32).wrapping_mul(280);
    (scaled >> 16) as u16
}

/// Generates a 16-bit "beat" ramp from a plain BPM value.
///
/// `bpm` may be a whole-number BPM (`< 256`, promoted to `Q8.8` by shifting
/// into the high byte) or an already-`Q8.8` value (`>= 256`) — see [`beat88`].
#[inline]
pub const fn beat16(bpm: u16, timebase: u32, now_millis: u32) -> u16 {
    let bpm88 = if bpm < 256 { bpm << 8 } else { bpm };
    beat88(bpm88, timebase, now_millis)
}

/// Generates an 8-bit "beat" ramp from a plain BPM value — see [`beat16`],
/// truncated to its high byte.
#[inline]
pub const fn beat8(bpm: u16, timebase: u32, now_millis: u32) -> u8 {
    (beat16(bpm, timebase, now_millis) >> 8) as u8
}

/// Generates a 16-bit sine wave oscillating at `bpm88` (a [`beat88`]-format
/// BPM), scaled to swing between `lowest` and `highest`.
///
/// `phase_offset` shifts the wave's phase (added to the beat ramp before
/// taking its sine — wraps at `65536`). Pass `lowest = 0, highest = 65535`
/// for the full output range.
#[inline]
pub const fn beatsin88(
    bpm88: u16,
    lowest: u16,
    highest: u16,
    timebase: u32,
    phase_offset: u16,
    now_millis: u32,
) -> u16 {
    let beat = beat88(bpm88, timebase, now_millis);
    let beatsin = (sin16(beat.wrapping_add(phase_offset)) as i32 + 32768) as u16;
    let rangewidth = highest.wrapping_sub(lowest);
    let scaledbeat = scale16(beatsin, rangewidth);
    lowest.wrapping_add(scaledbeat)
}

/// Generates a 16-bit sine wave oscillating at `bpm` (a [`beat16`]-format
/// BPM), scaled to swing between `lowest` and `highest` — see [`beatsin88`].
#[inline]
pub const fn beatsin16(
    bpm: u16,
    lowest: u16,
    highest: u16,
    timebase: u32,
    phase_offset: u16,
    now_millis: u32,
) -> u16 {
    let beat = beat16(bpm, timebase, now_millis);
    let beatsin = (sin16(beat.wrapping_add(phase_offset)) as i32 + 32768) as u16;
    let rangewidth = highest.wrapping_sub(lowest);
    let scaledbeat = scale16(beatsin, rangewidth);
    lowest.wrapping_add(scaledbeat)
}

/// Generates an 8-bit sine wave oscillating at `bpm` (a [`beat8`]-format
/// BPM), scaled to swing between `lowest` and `highest`.
///
/// `phase_offset` shifts the wave's phase (added to the beat ramp before
/// taking its sine — wraps at `256`). Pass `lowest = 0, highest = 255` for
/// the full output range.
#[inline]
pub const fn beatsin8(
    bpm: u16,
    lowest: u8,
    highest: u8,
    timebase: u32,
    phase_offset: u8,
    now_millis: u32,
) -> u8 {
    let beat = beat8(bpm, timebase, now_millis);
    let beatsin = sin8(beat.wrapping_add(phase_offset));
    let rangewidth = highest.wrapping_sub(lowest);
    let scaledbeat = scale8(beatsin, rangewidth);
    lowest.wrapping_add(scaledbeat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat88_ramps_and_wraps_with_time() {
        assert_eq!(beat88(120 << 8, 0, 0), 0);
        // 250 * (120<<8) * 280 == 2_150_400_000; >> 16 == 32812 (a beat at
        // 120 BPM lasts 60000/120 = 500ms, so this is ~halfway through it).
        assert_eq!(beat88(120 << 8, 0, 250), 32812);
        // 500 * (120<<8) * 280 == 4_300_800_000, which overflows u32 and
        // wraps to 5_832_704 before the shift — exactly like the original
        // C, which computes this chain in 32-bit and relies on the wrap.
        assert_eq!(beat88(120 << 8, 0, 500), 89);
    }

    #[test]
    fn beat16_promotes_plain_bpm_to_q88() {
        assert_eq!(beat16(120, 0, 250), beat16(120 << 8, 0, 250));
        assert_eq!(beat16(120, 0, 250), beat88(120 << 8, 0, 250));
    }

    #[test]
    fn beat8_is_high_byte_of_beat16() {
        for ms in (0..=2000u32).step_by(37) {
            assert_eq!(beat8(120, 0, ms), (beat16(120, 0, ms) >> 8) as u8);
        }
    }

    #[test]
    fn timebase_shifts_phase() {
        // Shifting the timebase forward by `d` is the same as looking `d`
        // milliseconds further into the future from timebase zero.
        assert_eq!(beat88(120 << 8, 1000, 1500), beat88(120 << 8, 0, 500));
    }

    #[test]
    fn beatsin_outputs_stay_within_their_range() {
        for ms in (0..=10_000u32).step_by(73) {
            let v8 = beatsin8(120, 10, 200, 0, 0, ms);
            assert!((10..=200).contains(&v8), "beatsin8 out of range: {v8}");

            let v16 = beatsin16(120, 1000, 60000, 0, 0, ms);
            assert!(
                (1000..=60000).contains(&v16),
                "beatsin16 out of range: {v16}"
            );

            let v88 = beatsin88(120 << 8, 1000, 60000, 0, 0, ms);
            assert!(
                (1000..=60000).contains(&v88),
                "beatsin88 out of range: {v88}"
            );
        }
    }

    #[test]
    fn beatsin_full_range_hits_both_ends() {
        let mut min8 = u8::MAX;
        let mut max8 = 0u8;
        for ms in (0..=2000u32).step_by(5) {
            let v = beatsin8(120, 0, 255, 0, 0, ms);
            min8 = min8.min(v);
            max8 = max8.max(v);
        }
        assert!(min8 < 10, "min8={min8}");
        assert!(max8 > 245, "max8={max8}");
    }

    #[test]
    fn phase_offset_shifts_the_wave() {
        let ms = 1234;
        let unshifted = beatsin8(120, 0, 255, 0, 0, ms);
        let shifted = beatsin8(120, 0, 255, 0, 128, ms);
        // A half-cycle phase offset should land on (roughly) the opposite
        // side of the wave.
        assert_ne!(unshifted, shifted);
    }
}
