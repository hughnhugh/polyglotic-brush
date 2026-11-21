// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! Half floats (IEEE 754 binary16), for seeding the simulation's textures.

/// The nearest half float to `value`, as its bits. Too large becomes
/// infinity; too small for a subnormal becomes zero; NaN stays NaN.
#[must_use]
pub fn from_f32(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;
    if exponent == 0xff {
        return sign | 0x7c00 | if mantissa != 0 { 0x0200 } else { 0 };
    }
    let unbiased = exponent - 127;
    if unbiased > 15 {
        return sign | 0x7c00;
    }
    if unbiased >= -14 {
        // Normal: keep 10 bits of mantissa, round to nearest, ties to even.
        let half = (((unbiased + 15) as u32) << 10) | (mantissa >> 13);
        let rest = mantissa & 0x1fff;
        let up = rest > 0x1000 || (rest == 0x1000 && half & 1 == 1);
        return sign | (half + u32::from(up)) as u16;
    }
    if unbiased >= -25 {
        // Subnormal: the implicit 1 comes back, shifted down.
        let full = mantissa | 0x0080_0000;
        let shift = (-unbiased - 14 + 13) as u32;
        let half = full >> shift;
        let rest = full & ((1 << shift) - 1);
        let mid = 1 << (shift - 1);
        let up = rest > mid || (rest == mid && half & 1 == 1);
        return sign | (half + u32::from(up)) as u16;
    }
    sign
}

#[cfg(test)]
mod tests {
    use super::from_f32;

    #[test]
    fn known_values() {
        assert_eq!(from_f32(0.0), 0x0000);
        assert_eq!(from_f32(-0.0), 0x8000);
        assert_eq!(from_f32(1.0), 0x3c00);
        assert_eq!(from_f32(-2.0), 0xc000);
        assert_eq!(from_f32(0.5), 0x3800);
        assert_eq!(from_f32(65504.0), 0x7bff);
        assert_eq!(from_f32(1.0e6), 0x7c00);
        assert_eq!(from_f32(f32::NEG_INFINITY), 0xfc00);
        assert_eq!(from_f32(f32::NAN) & 0x7c00, 0x7c00);
        assert_ne!(from_f32(f32::NAN) & 0x03ff, 0);
    }

    #[test]
    fn a_third_rounds_to_nearest() {
        // 1/3 = 0x3555 in half precision.
        assert_eq!(from_f32(1.0 / 3.0), 0x3555);
    }

    #[test]
    fn subnormals_and_underflow() {
        // The smallest subnormal half is 2^-24.
        assert_eq!(from_f32(2.0_f32.powi(-24)), 0x0001);
        assert_eq!(from_f32(2.0_f32.powi(-15)), 0x0200);
        assert_eq!(from_f32(1.0e-10), 0x0000);
    }
}
