// Reference implementations transcribed verbatim from FastLED's portable-C
// `lib8tion` sources, used ONLY to differentially test the `lib8tion` Rust
// crate's port against FastLED's actual behavior.
//
// Sources (as of the commit these were pulled from):
//   - src/platforms/shared/math8.h   (qadd8, blend8, mod8, ...)
//   - src/platforms/math8.h          (sqrt16, sqrt8)
//   - src/platforms/shared/scale8.h  (scale8, scale16, ...)
//   - src/platforms/shared/trig8.h   (sin16, sin8, ...)
//   - src/lib8tion.h                 (lerp*, map8, ease*, *wave8)
//
// All `#if FASTLED_SCALE8_FIXED == 1` branches are taken (that's the
// project's documented default), and the `*_AVRASM` assembly fast paths are
// skipped — exactly the portable-C surface the Rust port targets.

#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;

// ---------------------------------------------------------------------------
// math8 — platforms/shared/math8.h
// ---------------------------------------------------------------------------

u8 fl_qadd8(u8 i, u8 j) {
    u32 t = i + j;
    if (t > 255) t = 255;
    return (u8)t;
}

i8 fl_qadd7(i8 i, i8 j) {
    i16 t = (i16)i + (i16)j;
    if (t > 127) t = 127;
    else if (t < -128) t = -128;
    return (i8)t;
}

u8 fl_qsub8(u8 i, u8 j) {
    int t = i - j;
    if (t < 0) t = 0;
    return (u8)t;
}

u8 fl_add8(u8 i, u8 j) {
    int t = i + j;
    return (u8)t;
}

u16 fl_add8to16(u8 i, u16 j) {
    u16 t = (u16)i + j;
    return t;
}

u8 fl_sub8(u8 i, u8 j) {
    int t = i - j;
    return (u8)t;
}

u8 fl_avg8(u8 i, u8 j) {
    return (u8)((i + j) >> 1);
}

u16 fl_avg16(u16 i, u16 j) {
    u32 tmp = i;
    tmp += j;
    return (u16)(tmp >> 1);
}

u8 fl_avg8r(u8 i, u8 j) {
    return (u8)((i + j + 1) >> 1);
}

u16 fl_avg16r(u16 i, u16 j) {
    u32 tmp = i;
    tmp += j;
    tmp += 1;
    return (u16)(tmp >> 1);
}

i8 fl_avg7(i8 i, i8 j) {
    return (i8)((i >> 1) + (j >> 1) + (i & 0x1));
}

i16 fl_avg15(i16 i, i16 j) {
    return (i16)((i >> 1) + (j >> 1) + (i & 0x1));
}

u8 fl_mul8(u8 i, u8 j) {
    return (u8)(((int)i * (int)j) & 0xFF);
}

u8 fl_qmul8(u8 i, u8 j) {
    unsigned p = (unsigned)i * (unsigned)j;
    if (p > 255) p = 255;
    return (u8)p;
}

i8 fl_abs8(i8 i) {
    if (i < 0) i = -i;
    return i;
}

u8 fl_blend8_8bit(u8 a, u8 b, u8 amountOfB) {
    u16 partial;
    partial = (u16)(a << 8);
    partial = (u16)(partial + (u16)(b * amountOfB));
    partial = (u16)(partial - (u16)(a * amountOfB));
    partial = (u16)(partial + 0x80);
    return (u8)(partial >> 8);
}

u8 fl_blend8_16bit(u8 a, u8 b, u8 amountOfB) {
    u32 partial;
    i16 delta = (i16)b - (i16)a;
    partial = ((u32)a << 16);
    partial += (u32)delta * amountOfB * 257;
    partial += 0x8000;
    return (u8)(partial >> 16);
}

u8 fl_blend8(u8 a, u8 b, u8 amountOfB) {
    return fl_blend8_16bit(a, b, amountOfB);
}

// FastLED's BLEND_FIXED + SCALE8_FIXED formula, taken from the 3.6.0 tag
// (src/lib8tion/math8.h, BLEND8_C branch). Upstream later replaced this
// with the 0x80-rounding variant above; both are kept because they
// disagree — e.g. blend8(0, 255, 255) is 255 here and 254 there.
u8 fl_blend8_8bit_full_range(u8 a, u8 b, u8 amountOfB) {
    u16 partial;
    partial = (u16)((a << 8) | b); // A*256 + B
    partial = (u16)(partial + (u16)(b * amountOfB));
    partial = (u16)(partial - (u16)(a * amountOfB));
    return (u8)(partial >> 8);
}

u8 fl_mod8(u8 a, u8 m) {
    while (a >= m) a -= m;
    return a;
}

u8 fl_addmod8(u8 a, u8 b, u8 m) {
    a = (u8)(a + b);
    while (a >= m) a -= m;
    return a;
}

u8 fl_submod8(u8 a, u8 b, u8 m) {
    a = (u8)(a - b);
    while (a >= m) a -= m;
    return a;
}

// ---------------------------------------------------------------------------
// math8 — platforms/math8.h (sqrt)
// ---------------------------------------------------------------------------

u8 fl_sqrt16(u16 x) {
    if (x <= 1) {
        return (u8)x;
    }

    u8 low = 1;
    u8 hi, mid;

    if (x > 7904) {
        hi = 255;
    } else {
        hi = (u8)((x >> 5) + 8);
    }

    do {
        mid = (u8)((low + hi) >> 1);
        if ((u16)(mid * mid) > x) {
            hi = (u8)(mid - 1);
        } else {
            if (mid == 255) {
                return 255;
            }
            low = (u8)(mid + 1);
        }
    } while (hi >= low);

    return (u8)(low - 1);
}

// FastLED's sqrt8(x) == sqrt16(map8_to_16(x)), and map8_to_16 is bit
// replication: u16(x) * 0x101.
u8 fl_sqrt8(u8 x) {
    return fl_sqrt16((u16)((u16)x * 0x101u));
}

// ---------------------------------------------------------------------------
// scale8 — platforms/shared/scale8.h (FASTLED_SCALE8_FIXED == 1 branch)
// ---------------------------------------------------------------------------

u8 fl_scale8(u8 i, u8 scale) {
    return (u8)((((u16)i) * (1 + (u16)scale)) >> 8);
}

u8 fl_scale8_video(u8 i, u8 scale) {
    u8 j = (u8)((((int)i * (int)scale) >> 8) + ((i && scale) ? 1 : 0));
    return j;
}

u16 fl_scale16by8(u16 i, u8 scale) {
    if (scale == 0) {
        return 0;
    }
    return (u16)(((u32)i * (1 + (u32)scale)) >> 8);
}

u16 fl_scale16(u16 i, u16 scale) {
    return (u16)(((u32)i * (1 + (u32)scale)) / 65536u);
}

u32 fl_scale32by8(u32 i, u8 scale) {
    if (scale == 0) {
        return 0;
    }
    return (u32)(((u64)i * (1 + (u64)scale)) >> 8);
}

// dim/brighten — defined directly via scale8/scale8_video in lib8tion.h
u8 fl_dim8_raw(u8 x) { return fl_scale8(x, x); }
u8 fl_dim8_video(u8 x) { return fl_scale8_video(x, x); }
u8 fl_dim8_lin(u8 x) {
    if (x & 0x80) return fl_scale8(x, x);
    return (u8)((x + 1) / 2);
}
u8 fl_brighten8_raw(u8 x) {
    u8 ix = (u8)(255 - x);
    return (u8)(255 - fl_scale8(ix, ix));
}
u8 fl_brighten8_video(u8 x) {
    u8 ix = (u8)(255 - x);
    return (u8)(255 - fl_scale8_video(ix, ix));
}
u8 fl_brighten8_lin(u8 x) {
    u8 ix = (u8)(255 - x);
    u8 out = (ix & 0x80) ? fl_scale8(ix, ix) : (u8)((ix + 1) / 2);
    return (u8)(255 - out);
}

// ---------------------------------------------------------------------------
// trig8 — platforms/shared/trig8.h
// ---------------------------------------------------------------------------

i16 fl_sin16(u16 theta) {
    static const u16 base[] = {0, 6393, 12539, 18204, 23170, 27245, 30273, 32137};
    static const u8 slope[] = {49, 48, 44, 38, 31, 23, 14, 4};

    u16 offset = (u16)((theta & 0x3FFF) >> 3);
    if (theta & 0x4000) offset = (u16)(2047 - offset);

    u8 section = (u8)(offset / 256);
    u16 b = base[section];
    u8 m = slope[section];

    u8 secoffset8 = (u8)((u8)offset / 2);

    u16 mx = (u16)(m * secoffset8);
    i16 y = (i16)(mx + b);

    if (theta & 0x8000) y = (i16)(-y);

    return y;
}

i16 fl_cos16(u16 theta) { return fl_sin16((u16)(theta + 16384)); }

static const u8 b_m16_interleave[] = {0, 49, 49, 41, 90, 27, 117, 10};

u8 fl_sin8(u8 theta) {
    u8 offset = theta;
    if (theta & 0x40) {
        offset = (u8)(255 - offset);
    }
    offset &= 0x3F;

    u8 secoffset = offset & 0x0F;
    if (theta & 0x40) ++secoffset;

    u8 section = (u8)(offset >> 4);
    u8 s2 = (u8)(section * 2);
    const u8 *p = b_m16_interleave;
    p += s2;
    u8 b = *p;
    ++p;
    u8 m16 = *p;

    u8 mx = (u8)((m16 * secoffset) >> 4);

    i8 y = (i8)(mx + b);
    if (theta & 0x80) y = (i8)(-y);

    y = (i8)(y + 128);

    return (u8)y;
}

u8 fl_cos8(u8 theta) { return fl_sin8((u8)(theta + 64)); }

// ---------------------------------------------------------------------------
// LinearInterpolation — lib8tion.h
// ---------------------------------------------------------------------------

u8 fl_lerp8by8(u8 a, u8 b, u8 frac) {
    u8 result;
    if (b > a) {
        u8 delta = (u8)(b - a);
        u8 scaled = fl_scale8(delta, frac);
        result = (u8)(a + scaled);
    } else {
        u8 delta = (u8)(a - b);
        u8 scaled = fl_scale8(delta, frac);
        result = (u8)(a - scaled);
    }
    return result;
}

u16 fl_lerp16by16(u16 a, u16 b, u16 frac) {
    u16 result;
    if (b > a) {
        u16 delta = (u16)(b - a);
        u16 scaled = fl_scale16(delta, frac);
        result = (u16)(a + scaled);
    } else {
        u16 delta = (u16)(a - b);
        u16 scaled = fl_scale16(delta, frac);
        result = (u16)(a - scaled);
    }
    return result;
}

u16 fl_lerp16by8(u16 a, u16 b, u8 frac) {
    u16 result;
    if (b > a) {
        u16 delta = (u16)(b - a);
        u16 scaled = fl_scale16by8(delta, frac);
        result = (u16)(a + scaled);
    } else {
        u16 delta = (u16)(a - b);
        u16 scaled = fl_scale16by8(delta, frac);
        result = (u16)(a - scaled);
    }
    return result;
}

i16 fl_lerp15by8(i16 a, i16 b, u8 frac) {
    i16 result;
    if (b > a) {
        u16 delta = (u16)(b - a);
        u16 scaled = fl_scale16by8(delta, frac);
        result = (i16)(a + scaled);
    } else {
        u16 delta = (u16)(a - b);
        u16 scaled = fl_scale16by8(delta, frac);
        result = (i16)(a - scaled);
    }
    return result;
}

i16 fl_lerp15by16(i16 a, i16 b, u16 frac) {
    i16 result;
    if (b > a) {
        u16 delta = (u16)(b - a);
        u16 scaled = fl_scale16(delta, frac);
        result = (i16)(a + scaled);
    } else {
        u16 delta = (u16)(a - b);
        u16 scaled = fl_scale16(delta, frac);
        result = (i16)(a - scaled);
    }
    return result;
}

u8 fl_map8(u8 in, u8 rangeStart, u8 rangeEnd) {
    u8 rangeWidth = (u8)(rangeEnd - rangeStart);
    u8 out = fl_scale8(in, rangeWidth);
    out = (u8)(out + rangeStart);
    return out;
}

// ---------------------------------------------------------------------------
// Easing & Waveform Generators — lib8tion.h
// ---------------------------------------------------------------------------

u8 fl_ease8InOutQuad(u8 i) {
    u8 j = i;
    if (j & 0x80) {
        j = (u8)(255 - j);
    }
    u8 jj = fl_scale8(j, j);
    u8 jj2 = (u8)(jj << 1);
    if (i & 0x80) {
        jj2 = (u8)(255 - jj2);
    }
    return jj2;
}

u16 fl_ease16InOutQuad(u16 i) {
    u16 j = i;
    if (j & 0x8000) {
        j = (u16)(65535 - j);
    }
    u16 jj = fl_scale16(j, j);
    u16 jj2 = (u16)(jj << 1);
    if (i & 0x8000) {
        jj2 = (u16)(65535 - jj2);
    }
    return jj2;
}

u8 fl_ease8InOutCubic(u8 i) {
    u8 ii = fl_scale8(i, i);
    u8 iii = fl_scale8(ii, i);

    u16 r1 = (u16)((3 * (u16)ii) - (2 * (u16)iii));

    u8 result = (u8)r1;
    if (r1 & 0x100) {
        result = 255;
    }
    return result;
}

u16 fl_ease16InOutCubic(u16 i) {
    u32 ii = fl_scale16(i, i);
    u32 iii = fl_scale16((u16)ii, i);

    u32 r1 = (u32)((3 * ii) - (2 * iii));

    if (r1 > 65535) {
        return 65535;
    }
    return (u16)r1;
}

u8 fl_ease8InOutApprox(u8 i) {
    if (i < 64) {
        i = (u8)(i / 2);
    } else if (i > (255 - 64)) {
        i = (u8)(255 - i);
        i = (u8)(i / 2);
        i = (u8)(255 - i);
    } else {
        i = (u8)(i - 64);
        i = (u8)(i + (i / 2));
        i = (u8)(i + 32);
    }
    return i;
}

u8 fl_triwave8(u8 in) {
    if (in & 0x80) {
        in = (u8)(255 - in);
    }
    u8 out = (u8)(in << 1);
    return out;
}

u8 fl_quadwave8(u8 in) { return fl_ease8InOutQuad(fl_triwave8(in)); }
u8 fl_cubicwave8(u8 in) { return fl_ease8InOutCubic(fl_triwave8(in)); }

u8 fl_squarewave8(u8 in, u8 pulsewidth) {
    if (in < pulsewidth || (pulsewidth == 255)) {
        return 255;
    }
    return 0;
}
