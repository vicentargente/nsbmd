use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

#[derive(Clone, Copy)]
pub struct Fixed1_0_9 {
    value: i16
}

impl Fixed1_0_9 {
    const _INTEGER_BITS: usize = 0;
    const FRACTIONAL_BITS: usize = 9;

    const FRACTIONAL_MASK: i16 = (1 << Self::FRACTIONAL_BITS) - 1; // 1FF
    const NUMBER_DATA_MASK: i16 = (1 << (Self::FRACTIONAL_BITS + 1)) - 1; // 3FF
    const VOID_DATA_MASK: i16 = !Self::NUMBER_DATA_MASK; // 0xFC00
    const SIGN_MASK: i16 = 1 << Self::FRACTIONAL_BITS; // 0x200

    pub fn from_i16(value: i16) -> Self {
        let masked = value & Fixed1_0_9::NUMBER_DATA_MASK;
        let value = Self::propagate_sign(masked);

        Fixed1_0_9 { value }
    }

    pub fn to_i16(&self) -> i16 {
        self.value
    }

    pub fn from_f32(value: f32) -> Self {
        let max = 1.0 - 1.0 / (1 << Self::FRACTIONAL_BITS) as f32;
        let clamped = value.clamp(-1.0, max);
        let fixed_value = (clamped * (1 << Self::FRACTIONAL_BITS) as f32) as i16;
        let value = Self::propagate_sign(fixed_value);
        Fixed1_0_9 { value }
    }

    pub fn to_f32(&self) -> f32 {
        self.value as f32 / (1 << Fixed1_0_9::FRACTIONAL_BITS) as f32
    }

    pub fn from_f64(value: f64) -> Self {
        let max = 1.0 - 1.0 / (1 << Self::FRACTIONAL_BITS) as f64;
        let clamped = value.clamp(-1.0, max);
        let fixed_value = (clamped * (1 << Self::FRACTIONAL_BITS) as f64) as i16;
        let value = Self::propagate_sign(fixed_value);
        Fixed1_0_9 { value }
    }

    pub fn to_f64(&self) -> f64 {
        self.value as f64 / (1 << Fixed1_0_9::FRACTIONAL_BITS) as f64
    }

    pub fn get_int(&self) -> i16 {
        self.value >> Fixed1_0_9::FRACTIONAL_BITS
    }

    pub fn get_frac(&self) -> i16 {
        self.value & Fixed1_0_9::FRACTIONAL_MASK
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.value.to_le_bytes()
    }

    fn propagate_sign(value: i16) -> i16 {
        if value & Fixed1_0_9::SIGN_MASK != 0 {
            value | Fixed1_0_9::VOID_DATA_MASK
        } else {
            value & !Fixed1_0_9::VOID_DATA_MASK
        }
    }

}

impl Add for Fixed1_0_9 {
    type Output = Fixed1_0_9;
    
    fn add(self, rhs: Self) -> Self::Output {
        let value = Self::propagate_sign(self.value + rhs.value);

        Fixed1_0_9 {
            value
        }
    }
}

impl Sub for Fixed1_0_9 {
    type Output = Fixed1_0_9;
    
    fn sub(self, rhs: Self) -> Self::Output {
        let value = Self::propagate_sign(self.value - rhs.value);

        Fixed1_0_9 {
            value
        }
    }
}

impl Mul for Fixed1_0_9 {
    type Output = Fixed1_0_9;
    
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        let value = Self::propagate_sign(((lhs_val * rhs_val) >> Fixed1_0_9::FRACTIONAL_BITS) as i16);

        Fixed1_0_9 {
            value
        }
    }
}

impl Div for Fixed1_0_9 {
    type Output = Fixed1_0_9;
    
    fn div(self, rhs: Self) -> Self::Output {
        // let lhs_val = self.value as i32;
        // let rhs_val = rhs.value as i32;

        // if rhs_val == 0 {
        //     panic!("Division by zero in Fixed1_0_9");
        // }

        // let num = lhs_val << Fixed1_0_9::FRACTIONAL_BITS;
        // let denom = rhs_val as i32;
        // let cocient = num / denom;

        // let value = Self::propagate_sign(((lhs_val << Fixed1_0_9::FRACTIONAL_BITS) / rhs_val) as i16);

        // Fixed1_0_9 {
        //     value
        // }

        if rhs.value == 0 {
            panic!("Division by zero in Fixed1_0_9");
        }
        
        Self::from_f32(self.to_f32() / rhs.to_f32())
    }
}

impl PartialEq for Fixed1_0_9 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Fixed1_0_9 {}

impl PartialOrd for Fixed1_0_9 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Fixed1_0_9 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Default for Fixed1_0_9 {
    fn default() -> Self {
        Fixed1_0_9 { value: 0 }
    }
}

impl From<i16> for Fixed1_0_9 {
    fn from(value: i16) -> Self {
        Fixed1_0_9::from_i16(value)
    }
}

impl Into<i16> for Fixed1_0_9 {
    fn into(self) -> i16 {
        self.to_i16()
    }
}

impl From<f32> for Fixed1_0_9 {
    fn from(value: f32) -> Self {
        Fixed1_0_9::from_f32(value)
    }
}

impl Into<f32> for Fixed1_0_9 {
    fn into(self) -> f32 {
        self.to_f32()
    }
}

impl From<f64> for Fixed1_0_9 {
    fn from(value: f64) -> Self {
        Fixed1_0_9::from_f64(value)
    }
}

impl Into<f64> for Fixed1_0_9 {
    fn into(self) -> f64 {
        self.to_f64()
    }
}

impl Debug for Fixed1_0_9 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = if self.value < 0 { "-" } else { "" };
        let abs_value = self.value.abs(); // Handle negation carefully

        let integer = abs_value >> Self::FRACTIONAL_BITS;
        let fractional = abs_value & Self::FRACTIONAL_MASK;

        let mut numerator = fractional as u32;
        let mut digits = String::with_capacity(Self::FRACTIONAL_BITS);

        // Generate each of the 9 decimal digits
        for _ in 0..Self::FRACTIONAL_BITS {
            numerator *= 10;
            let digit = (numerator >> Self::FRACTIONAL_BITS) as u8;
            digits.push(char::from_digit(digit.into(), 10).unwrap());
            numerator &= Self::FRACTIONAL_MASK as u32;
        }

        // Trim trailing zeros, but ensure at least one digit remains
        let trimmed = digits.trim_end_matches('0');
        let fractional_str = if trimmed.is_empty() {
            "0"
        } else {
            trimmed
        };

        write!(f, "Fixed1_0_9({}{}.{})", sign, integer, fractional_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::EPSILON;

    #[test]
    fn test_from_i16_max_positive() {
        let fixed = Fixed1_0_9::from_i16(0x1FF);
        assert_eq!(fixed.to_f32(), 0.998046875);
    }

    #[test]
    fn test_from_i16_min_negative() {
        let fixed = Fixed1_0_9::from_i16(0x200);
        assert_eq!(fixed.to_f32(), -1.0);
    }

    #[test]
    fn test_from_f32_clamping() {
        let fixed = Fixed1_0_9::from_f32(1.5);
        assert_eq!(fixed.to_f32(), 0.998046875); // Clamped to max
        let fixed_neg = Fixed1_0_9::from_f32(-1.5);
        assert_eq!(fixed_neg.to_f32(), -1.0); // Clamped to min
    }

    #[test]
    fn test_add_basic() {
        let a = Fixed1_0_9::from_f32(0.5);
        let b = Fixed1_0_9::from_f32(0.25);
        let sum = a + b;
        assert!((sum.to_f32() - 0.75).abs() < EPSILON);
    }

    #[test]
    fn test_add_overflow() {
        let a = Fixed1_0_9::from_f32(0.998046875); // Max positive
        let b = Fixed1_0_9::from_f32(0.001953125);  // 2 ^ -9
        let sum = a + b;
        // Expect overflow to -1.0
        assert_eq!(sum.to_f32(), -1.0);
    }

    #[test]
    fn test_sub_basic() {
        let a = Fixed1_0_9::from_f32(0.75);
        let b = Fixed1_0_9::from_f32(0.25);
        let diff = a - b;
        assert!((diff.to_f32() - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_sub_underflow() {
        let a = Fixed1_0_9::from_f32(-1.0); // Min negative
        let b = Fixed1_0_9::from_f32(0.001953125);
        let diff = a - b;
        // (min_negative - smallest positive) Expect underflow to max value
        assert_eq!(diff.to_f32(), 0.998046875);
    }

    #[test]
    fn test_mul_basic() {
        let a = Fixed1_0_9::from_f32(0.5);
        let b = Fixed1_0_9::from_f32(0.5);
        let product = a * b;
        assert!((product.to_f32() - 0.25).abs() < EPSILON);
    }

    #[test]
    fn test_div_basic() {
        let a = Fixed1_0_9::from_f32(0.5);
        let b = Fixed1_0_9::from_f32(0.25);
        let quotient = a / b;
        // Expect 2.0, but clamped to max (0.998)
        assert_eq!(quotient.to_f32(), 0.998046875);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero() {
        let a = Fixed1_0_9::from_f32(0.5);
        let b = Fixed1_0_9::from_f32(0.0);
        let _ = a / b;
    }

    #[test]
    fn test_debug_format() {
        let fixed = Fixed1_0_9::from_f32(0.998046875);
        assert_eq!(format!("{:?}", fixed), "Fixed1_0_9(0.998046875)");
        let fixed_neg = Fixed1_0_9::from_f32(-1.0);
        assert_eq!(format!("{:?}", fixed_neg), "Fixed1_0_9(-1.0)");
    }

    #[test]
    fn test_round_trip_f32() {
        let value = 0.123456789;
        let fixed = Fixed1_0_9::from_f32(value);
        let converted = fixed.to_f32();
        // Check truncation/rounding
        let expected = (value * 512.0).trunc() / 512.0;
        assert_eq!(converted, expected);
    }

    #[test]
    fn test_get_int_and_frac() {
        let fixed = Fixed1_0_9::from_f32(0.75390625); // 0.75390625 *512 = 386 (0x182)
        assert_eq!(fixed.get_int(), 0);
        assert_eq!(fixed.get_frac(), 386);
    }
}
