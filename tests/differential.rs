//! Differential tests: assert that `lib8tion`'s pure-Rust port produces
//! bit-for-bit identical output to FastLED's actual portable-C reference
//! implementation (compiled and linked via the `fastled-ref` helper crate).
//!
//! Inputs are exhaustively swept for single-byte domains, and sampled across
//! representative ranges (plus boundary values) for wider domains where an
//! exhaustive sweep would be too slow.

use lib8tion as l;

fn u16_samples() -> impl Iterator<Item = u16> {
    let boundaries = [
        0u16, 1, 2, 127, 128, 255, 256, 257, 1000, 0x3FFF, 0x4000, 0x7FFF, 0x8000, 0xC000, 0xFFFE,
        0xFFFF,
    ];
    boundaries
        .into_iter()
        .chain((0..=255u32).map(|x| (x * 257) as u16))
}

fn i16_samples() -> impl Iterator<Item = i16> {
    let boundaries = [
        i16::MIN,
        i16::MIN + 1,
        -1000,
        -1,
        0,
        1,
        1000,
        i16::MAX - 1,
        i16::MAX,
    ];
    boundaries
        .into_iter()
        .chain((0..=255u32).map(|x| (x * 257) as i32 as i16))
}

fn u32_samples() -> impl Iterator<Item = u32> {
    [
        0u32,
        1,
        2,
        0xFF,
        0x100,
        0xFFFF,
        0x1_0000,
        0x00FF_FFFF,
        0x0100_0000,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFE,
        0xFFFF_FFFF,
    ]
    .into_iter()
}

#[test]
fn math8_byte_pairs_exhaustive() {
    for i in 0..=255u8 {
        for j in [0u8, 1, 2, 3, 7, 16, 31, 63, 64, 127, 128, 200, 254, 255]
            .into_iter()
            .chain(0..=255u8)
        {
            assert_eq!(l::qadd8(i, j), fastled_ref::qadd8(i, j), "qadd8({i},{j})");
            assert_eq!(l::qsub8(i, j), fastled_ref::qsub8(i, j), "qsub8({i},{j})");
            assert_eq!(l::add8(i, j), fastled_ref::add8(i, j), "add8({i},{j})");
            assert_eq!(l::sub8(i, j), fastled_ref::sub8(i, j), "sub8({i},{j})");
            assert_eq!(l::avg8(i, j), fastled_ref::avg8(i, j), "avg8({i},{j})");
            assert_eq!(l::avg8r(i, j), fastled_ref::avg8r(i, j), "avg8r({i},{j})");
            assert_eq!(l::mul8(i, j), fastled_ref::mul8(i, j), "mul8({i},{j})");
            assert_eq!(l::qmul8(i, j), fastled_ref::qmul8(i, j), "qmul8({i},{j})");
            assert_eq!(
                l::add8to16(i, j as u16),
                fastled_ref::add8to16(i, j as u16),
                "add8to16({i},{j})"
            );
            assert_eq!(
                l::scale8(i, j),
                fastled_ref::scale8(i, j),
                "scale8({i},{j})"
            );
            assert_eq!(
                l::scale8_video(i, j),
                fastled_ref::scale8_video(i, j),
                "scale8_video({i},{j})"
            );
            assert_eq!(
                l::blend8(i, j, j),
                fastled_ref::blend8(i, j, j),
                "blend8({i},{j},{j})"
            );
            assert_eq!(
                l::blend8_8bit(i, j, j),
                fastled_ref::blend8_8bit(i, j, j),
                "blend8_8bit({i},{j},{j})"
            );
            assert_eq!(
                l::blend8_16bit(i, j, j),
                fastled_ref::blend8_16bit(i, j, j),
                "blend8_16bit({i},{j},{j})"
            );
            assert_eq!(
                l::lerp8by8(i, j, j),
                fastled_ref::lerp8by8(i, j, j),
                "lerp8by8({i},{j},{j})"
            );
        }
    }
}

#[test]
fn math8_signed_byte_pairs_exhaustive() {
    for i in i8::MIN..=i8::MAX {
        for j in i8::MIN..=i8::MAX {
            assert_eq!(l::qadd7(i, j), fastled_ref::qadd7(i, j), "qadd7({i},{j})");
            assert_eq!(l::avg7(i, j), fastled_ref::avg7(i, j), "avg7({i},{j})");
            if j != i8::MIN {
                continue;
            }
        }
        assert_eq!(l::abs8(i), fastled_ref::abs8(i), "abs8({i})");
    }
}

#[test]
fn mod8_family_exhaustive() {
    for a in 0..=255u8 {
        for m in 1..=255u8 {
            assert_eq!(l::mod8(a, m), fastled_ref::mod8(a, m), "mod8({a},{m})");
            for b in [0u8, 1, 7, 100, 255] {
                assert_eq!(
                    l::addmod8(a, b, m),
                    fastled_ref::addmod8(a, b, m),
                    "addmod8({a},{b},{m})"
                );
                assert_eq!(
                    l::submod8(a, b, m),
                    fastled_ref::submod8(a, b, m),
                    "submod8({a},{b},{m})"
                );
            }
        }
    }
}

#[test]
fn avg16_family_and_sqrt() {
    for x in u16_samples() {
        assert_eq!(l::sqrt16(x), fastled_ref::sqrt16(x), "sqrt16({x})");
        for y in u16_samples() {
            assert_eq!(l::avg16(x, y), fastled_ref::avg16(x, y), "avg16({x},{y})");
            assert_eq!(
                l::avg16r(x, y),
                fastled_ref::avg16r(x, y),
                "avg16r({x},{y})"
            );
        }
    }
    for x in 0..=255u8 {
        assert_eq!(l::sqrt8(x), fastled_ref::sqrt8(x), "sqrt8({x})");
    }
}

#[test]
fn avg15_signed() {
    for x in i16_samples() {
        for y in i16_samples() {
            assert_eq!(l::avg15(x, y), fastled_ref::avg15(x, y), "avg15({x},{y})");
        }
    }
}

#[test]
fn scale8_variants_and_dim_brighten_exhaustive() {
    for i in 0..=255u8 {
        assert_eq!(l::dim8_raw(i), fastled_ref::dim8_raw(i), "dim8_raw({i})");
        assert_eq!(
            l::dim8_video(i),
            fastled_ref::dim8_video(i),
            "dim8_video({i})"
        );
        assert_eq!(l::dim8_lin(i), fastled_ref::dim8_lin(i), "dim8_lin({i})");
        assert_eq!(
            l::brighten8_raw(i),
            fastled_ref::brighten8_raw(i),
            "brighten8_raw({i})"
        );
        assert_eq!(
            l::brighten8_video(i),
            fastled_ref::brighten8_video(i),
            "brighten8_video({i})"
        );
        assert_eq!(
            l::brighten8_lin(i),
            fastled_ref::brighten8_lin(i),
            "brighten8_lin({i})"
        );
        for scale in 0..=255u8 {
            assert_eq!(
                l::scale8_constexpr(i, scale),
                fastled_ref::scale8(i, scale),
                "scale8_constexpr({i},{scale})"
            );
        }
    }
}

#[test]
fn scale16_and_friends() {
    for i in u16_samples() {
        for scale in [0u16, 1, 2, 0x7FFF, 0x8000, 0xFFFF]
            .into_iter()
            .chain((0..=255u32).map(|x| (x * 257) as u16))
        {
            assert_eq!(
                l::scale16(i, scale),
                fastled_ref::scale16(i, scale),
                "scale16({i},{scale})"
            );
        }
        for scale in 0..=255u8 {
            assert_eq!(
                l::scale16by8(i, scale),
                fastled_ref::scale16by8(i, scale),
                "scale16by8({i},{scale})"
            );
            assert_eq!(
                l::lerp16by8(i, i.wrapping_add(12345), scale),
                fastled_ref::lerp16by8(i, i.wrapping_add(12345), scale),
                "lerp16by8({i},_,{scale})"
            );
        }
    }
    for i in u32_samples() {
        for scale in 0..=255u8 {
            assert_eq!(
                l::scale32by8(i, scale),
                fastled_ref::scale32by8(i, scale),
                "scale32by8({i},{scale})"
            );
        }
    }
}

#[test]
fn lerp16by16_and_lerp15_variants() {
    for a in u16_samples() {
        let b = a.wrapping_add(0x5A5A);
        for frac in [0u16, 1, 0x7FFF, 0x8000, 0xFFFF]
            .into_iter()
            .chain((0..=255u32).map(|x| (x * 257) as u16))
        {
            assert_eq!(
                l::lerp16by16(a, b, frac),
                fastled_ref::lerp16by16(a, b, frac),
                "lerp16by16({a},{b},{frac})"
            );
        }
    }
    for a in i16_samples() {
        let b = a.wrapping_add(12345);
        for frac in 0..=255u8 {
            assert_eq!(
                l::lerp15by8(a, b, frac),
                fastled_ref::lerp15by8(a, b, frac),
                "lerp15by8({a},{b},{frac})"
            );
        }
        for frac in [0u16, 1, 0x7FFF, 0x8000, 0xFFFF] {
            assert_eq!(
                l::lerp15by16(a, b, frac),
                fastled_ref::lerp15by16(a, b, frac),
                "lerp15by16({a},{b},{frac})"
            );
        }
    }
}

#[test]
fn map8_exhaustive_bounded_ranges() {
    for input in 0..=255u8 {
        for start in 0..=255u8 {
            for end in start..=255u8 {
                assert_eq!(
                    l::map8(input, start, end),
                    fastled_ref::map8(input, start, end),
                    "map8({input},{start},{end})"
                );
            }
        }
    }
}

#[test]
fn trig8_exhaustive() {
    for theta in 0..=255u8 {
        assert_eq!(l::sin8(theta), fastled_ref::sin8(theta), "sin8({theta})");
        assert_eq!(l::cos8(theta), fastled_ref::cos8(theta), "cos8({theta})");
    }
    for theta in u16_samples() {
        assert_eq!(l::sin16(theta), fastled_ref::sin16(theta), "sin16({theta})");
        assert_eq!(l::cos16(theta), fastled_ref::cos16(theta), "cos16({theta})");
    }
    // sin16/cos16 are sampled densely too, since the reference uses a
    // piecewise-linear lookup table whose section boundaries matter.
    for theta in (0..=u16::MAX).step_by(37) {
        assert_eq!(l::sin16(theta), fastled_ref::sin16(theta), "sin16({theta})");
        assert_eq!(l::cos16(theta), fastled_ref::cos16(theta), "cos16({theta})");
    }
}

#[test]
fn ease_and_waveforms_exhaustive() {
    for i in 0..=255u8 {
        assert_eq!(
            l::ease8_in_out_quad(i),
            fastled_ref::ease8_in_out_quad(i),
            "ease8_in_out_quad({i})"
        );
        assert_eq!(
            l::ease8_in_out_cubic(i),
            fastled_ref::ease8_in_out_cubic(i),
            "ease8_in_out_cubic({i})"
        );
        assert_eq!(
            l::ease8_in_out_approx(i),
            fastled_ref::ease8_in_out_approx(i),
            "ease8_in_out_approx({i})"
        );
        assert_eq!(l::triwave8(i), fastled_ref::triwave8(i), "triwave8({i})");
        assert_eq!(l::quadwave8(i), fastled_ref::quadwave8(i), "quadwave8({i})");
        assert_eq!(
            l::cubicwave8(i),
            fastled_ref::cubicwave8(i),
            "cubicwave8({i})"
        );
        for pulsewidth in [0u8, 1, 64, 127, 128, 200, 254, 255] {
            assert_eq!(
                l::squarewave8(i, pulsewidth),
                fastled_ref::squarewave8(i, pulsewidth),
                "squarewave8({i},{pulsewidth})"
            );
        }
    }
    for i in (0..=u16::MAX).step_by(31) {
        assert_eq!(
            l::ease16_in_out_quad(i),
            fastled_ref::ease16_in_out_quad(i),
            "ease16_in_out_quad({i})"
        );
        assert_eq!(
            l::ease16_in_out_cubic(i),
            fastled_ref::ease16_in_out_cubic(i),
            "ease16_in_out_cubic({i})"
        );
    }
}
