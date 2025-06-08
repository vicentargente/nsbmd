use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

#[derive(Clone, Copy)]
pub struct Fixed1_3_12 {
    value: i16
}

impl Fixed1_3_12 {
    const _INTEGER_BITS: usize = 3;
    const FRACTIONAL_BITS: usize = 12;
    const FRACTIONAL_MASK: i16 = (1 << Self::FRACTIONAL_BITS) - 1;

    pub fn from_i16(value: i16) -> Self {
        Fixed1_3_12 { value }
    }

    pub fn to_i16(&self) -> i16 {
        self.value
    }

    pub fn from_f32(value: f32) -> Self {
        let fixed_value = (value * (1 << Fixed1_3_12::FRACTIONAL_BITS) as f32) as i16;
        Fixed1_3_12 { value: fixed_value }
    }

    pub fn to_f32(&self) -> f32 {
        self.value as f32 / (1 << Fixed1_3_12::FRACTIONAL_BITS) as f32
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_value = (value * (1 << Fixed1_3_12::FRACTIONAL_BITS) as f64) as i16;
        Fixed1_3_12 { value: fixed_value }
    }

    pub fn to_f64(&self) -> f64 {
        self.value as f64 / (1 << Fixed1_3_12::FRACTIONAL_BITS) as f64
    }

    pub fn get_int(&self) -> i16 {
        self.value >> Fixed1_3_12::FRACTIONAL_BITS
    }

    pub fn get_frac(&self) -> i16 {
        self.value & Fixed1_3_12::FRACTIONAL_MASK
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.value.to_le_bytes()
    }
}

impl Add for Fixed1_3_12 {
    type Output = Fixed1_3_12;
    
    fn add(self, rhs: Self) -> Self::Output {
        Fixed1_3_12 {
            value: self.value.wrapping_add(rhs.value)
        }
    }
}

impl Sub for Fixed1_3_12 {
    type Output = Fixed1_3_12;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Fixed1_3_12 {
            value: self.value.wrapping_sub(rhs.value)
        }
    }
}

impl Mul for Fixed1_3_12 {
    type Output = Fixed1_3_12;
    
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        Fixed1_3_12 {
            value: ((lhs_val * rhs_val) >> Fixed1_3_12::FRACTIONAL_BITS) as i16
        }
    }
}

impl Div for Fixed1_3_12 {
    type Output = Fixed1_3_12;
    
    fn div(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        if rhs_val == 0 {
            panic!("Division by zero in Fixed1_3_12");
        }

        Fixed1_3_12 {
            value: ((lhs_val << Fixed1_3_12::FRACTIONAL_BITS) / rhs_val) as i16
        }
    }
}

impl PartialEq for Fixed1_3_12 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Fixed1_3_12 {}

impl PartialOrd for Fixed1_3_12 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Fixed1_3_12 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Default for Fixed1_3_12 {
    fn default() -> Self {
        Fixed1_3_12 { value: 0 }
    }
}

impl From<i16> for Fixed1_3_12 {
    fn from(value: i16) -> Self {
        Fixed1_3_12::from_i16(value)
    }
}

impl Into<i16> for Fixed1_3_12 {
    fn into(self) -> i16 {
        self.to_i16()
    }
}

impl From<f32> for Fixed1_3_12 {
    fn from(value: f32) -> Self {
        Fixed1_3_12::from_f32(value)
    }
}

impl Into<f32> for Fixed1_3_12 {
    fn into(self) -> f32 {
        self.to_f32()
    }
}

impl From<f64> for Fixed1_3_12 {
    fn from(value: f64) -> Self {
        Fixed1_3_12::from_f64(value)
    }
}

impl Into<f64> for Fixed1_3_12 {
    fn into(self) -> f64 {
        self.to_f64()
    }
}

impl Debug for Fixed1_3_12 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.value;
        let type_name = stringify!(Fixed1_3_12);

        let sign_str = if val < 0 { "-" } else { "" };

        let display_integer: i16;
        let fractional_numerator: u32;

        if val == i16::MIN {
            display_integer = (val >> Self::FRACTIONAL_BITS).wrapping_abs();
            fractional_numerator = (val & Self::FRACTIONAL_MASK) as u32;
        } else {
            let abs_val = val.abs();
            display_integer = abs_val >> Self::FRACTIONAL_BITS;
            fractional_numerator = (abs_val & Self::FRACTIONAL_MASK) as u32;
        }

        let mut current_numerator = fractional_numerator;
        let mut digits = String::with_capacity(Self::FRACTIONAL_BITS);


        if current_numerator != 0 {
            for _ in 0..Self::FRACTIONAL_BITS {
                current_numerator *= 10;
                let digit = (current_numerator >> Self::FRACTIONAL_BITS) as u8;
                digits.push(char::from_digit(digit.into(), 10).unwrap_or('0'));
                current_numerator &= (1 << Self::FRACTIONAL_BITS) -1;
            }
        }

        let trimmed_digits = digits.trim_end_matches('0');
        let fractional_str = if trimmed_digits.is_empty() {
            "0"
        } else {
            trimmed_digits
        };

        write!(f, "{}({}{}.{})", type_name, sign_str, display_integer, fractional_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001; // For f32 comparisons
    const EPSILON_F64: f64 = 0.0000001; // For f64 comparisons

    fn assert_f32_eq(a: f32, b: f32, msg: &str) {
        assert!((a - b).abs() < EPSILON, "{} - Expected: {}, Got: {}", msg, b, a);
    }

    fn assert_f64_eq(a: f64, b: f64, msg: &str) {
        assert!((a - b).abs() < EPSILON_F64, "{} - Expected: {}, Got: {}", msg, b, a);
    }

    #[test]
    fn test_from_to_f32() {
        let val1 = Fixed1_3_12::from_f32(2.5);
        assert_f32_eq(val1.to_f32(), 2.5, "Positive value");

        let val2 = Fixed1_3_12::from_f32(-1.75);
        assert_f32_eq(val2.to_f32(), -1.75, "Negative value");

        let val3 = Fixed1_3_12::from_f32(0.0);
        assert_f32_eq(val3.to_f32(), 0.0, "Zero value");

        // Max positive representable integer part is 7
        let val4 = Fixed1_3_12::from_f32(7.999); // close to max
        assert_f32_eq(val4.to_f32(), 32764_i16 as f32 / 4096.0, "Near max positive value");


        let val5 = Fixed1_3_12::from_f32(-8.0); // min value
        assert_f32_eq(val5.to_f32(), -8.0, "Min negative value (-8.0)");
    }

    #[test]
    fn test_from_to_f64() {
        let val1 = Fixed1_3_12::from_f64(2.5);
        assert_f64_eq(val1.to_f64(), 2.5, "Positive value f64");

        let val2 = Fixed1_3_12::from_f64(-1.75);
        assert_f64_eq(val2.to_f64(), -1.75, "Negative value f64");
    }

    #[test]
    fn test_addition() {
        let a = Fixed1_3_12::from_f32(1.5);
        let b = Fixed1_3_12::from_f32(2.25);
        assert_f32_eq((a + b).to_f32(), 3.75, "Addition");
    }

    #[test]
    fn test_subtraction() {
        let a = Fixed1_3_12::from_f32(3.75);
        let b = Fixed1_3_12::from_f32(1.5);
        assert_f32_eq((a - b).to_f32(), 2.25, "Subtraction");
    }

    #[test]
    fn test_multiplication() {
        let a = Fixed1_3_12::from_f32(2.0);
        let b = Fixed1_3_12::from_f32(1.5);
        assert_f32_eq((a * b).to_f32(), 3.0, "Multiplication");

        let c = Fixed1_3_12::from_f32(-2.0);
        let d = Fixed1_3_12::from_f32(1.5);
        assert_f32_eq((c * d).to_f32(), -3.0, "Multiplication with negative");
    }

    #[test]
    fn test_division() {
        let a = Fixed1_3_12::from_f32(3.0);
        let b = Fixed1_3_12::from_f32(2.0);
        assert_f32_eq((a / b).to_f32(), 1.5, "Division");

        let c = Fixed1_3_12::from_f32(-3.0);
        let d = Fixed1_3_12::from_f32(2.0);
        assert_f32_eq((c / d).to_f32(), -1.5, "Division with negative");
    }

    #[test]
    #[should_panic(expected = "Division by zero in Fixed1_3_12")]
    fn test_division_by_zero() {
        let a = Fixed1_3_12::from_f32(1.0);
        let b = Fixed1_3_12::from_f32(0.0);
        let _ = a / b;
    }
    
    #[test]
    fn test_debug_format() {
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(0.0)), "Fixed1_3_12(0.0)");
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(1.0)), "Fixed1_3_12(1.0)");
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(-1.0)), "Fixed1_3_12(-1.0)");
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(2.5)), "Fixed1_3_12(2.5)"); // 2 + 2048/4096
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(-2.5)), "Fixed1_3_12(-2.5)");
        
        // 0.125 = 512/4096
        assert_eq!(format!("{:?}", Fixed1_3_12::from_f32(0.125)), "Fixed1_3_12(0.125)");
        // 0.000244140625 = 1/4096 (smallest positive fraction)
        let smallest_pos_frac = Fixed1_3_12::from_i16(1); // raw value 1
        assert_eq!(format!("{:?}", smallest_pos_frac), "Fixed1_3_12(0.000244140625)");

        // Test -8.0 (i16::MIN)
        let neg_eight = Fixed1_3_12::from_i16(i16::MIN);
        assert_eq!(neg_eight.to_f32(), -8.0, "Value check for -8.0");
        assert_eq!(format!("{:?}", neg_eight), "Fixed1_3_12(-8.0)");

        // Test max positive value (7.999755859375)
        let max_pos = Fixed1_3_12::from_i16(i16::MAX); // 32767
        assert_eq!(format!("{:?}", max_pos), "Fixed1_3_12(7.999755859375)");

        // Test value just below -7.0
        let near_neg_seven = Fixed1_3_12::from_f32(-7.000244140625); // -7 - 1/4096
        assert_eq!(format!("{:?}", near_neg_seven), "Fixed1_3_12(-7.000244140625)");
    }

    #[test]
    fn test_get_int_frac() {
        let a = Fixed1_3_12::from_f32(3.75); // 3 * 4096 + 0.75 * 4096 = 12288 + 3072 = 15360
        assert_eq!(a.get_int(), 3);
        assert_eq!(a.get_frac(), 3072); // 0.75 * 4096

        let b = Fixed1_3_12::from_f32(-3.75);
        // For negative numbers, get_int is floor, get_frac is positive offset from that floor * scale
        // -3.75 internal value: (-3.75 * 4096) = -15360
        // get_int: -15360 >> 12 = -4 (due to sign extension and right shift behavior for negatives)
        // get_frac: -15360 & 0xFFF = (-15360 & 4095)
        // -15360 in binary (16-bit two's complement): 1100010000000000
        // 0xFFF in binary (16-bit):                 0000111111111111
        // AND result:                               0000010000000000 = 1024
        // This means -3.75 = -4 + (1024/4096) = -4 + 0.25
        assert_eq!(b.get_int(), -4);
        assert_eq!(b.get_frac(), 1024);

        let c = Fixed1_3_12::from_f32(-8.0); // i16::MIN = -32768
        assert_eq!(c.get_int(), -8);
        assert_eq!(c.get_frac(), 0);
    }

    #[test]
    fn test_overflow_behavior() {
        // Max positive value is approx 7.99975
        let max_val = Fixed1_3_12::from_i16(i16::MAX); // 7.999755859375
        let _one = Fixed1_3_12::from_f32(1.0);
        let smallest_frac = Fixed1_3_12::from_i16(1); // Smallest positive fraction

        // Adding smallest fraction to max_val should overflow to negative
        // i16::MAX + 1 = i16::MIN
        let overflow_add = max_val + smallest_frac;
        assert_eq!(overflow_add.to_i16(), i16::MIN, "Addition overflow to i16::MIN");
        assert_f32_eq(overflow_add.to_f32(), -8.0, "Addition overflow check");

        // Multiplication overflow
        let four = Fixed1_3_12::from_f32(4.0);
        let three = Fixed1_3_12::from_f32(3.0); // 4.0 * 3.0 = 12.0, which is out of range
        // (4*4096) * (3*4096) >> 12 = 16384 * 12288 >> 12
        // 201326592 >> 12 = 49152. This as i16 is -16384 (wraps around)
        // -16384 / 4096.0 = -4.0
        let mul_overflow = four * three;
        assert_f32_eq(mul_overflow.to_f32(), -4.0, "Multiplication overflow check");
    }
}
