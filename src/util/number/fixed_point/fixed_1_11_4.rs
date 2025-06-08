use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

#[derive(Clone, Copy)]
pub struct Fixed1_11_4 {
    value: i16
}

impl Fixed1_11_4 {
    const _INTEGER_BITS: usize = 11;
    const FRACTIONAL_BITS: usize = 4;
    const FRACTIONAL_MASK: i16 = (1 << Self::FRACTIONAL_BITS) - 1;

    pub fn from_i16(value: i16) -> Self {
        Fixed1_11_4 { value }
    }

    pub fn to_i16(&self) -> i16 {
        self.value
    }

    pub fn from_f32(value: f32) -> Self {
        let fixed_value = (value * (1 << Fixed1_11_4::FRACTIONAL_BITS) as f32) as i16;
        Fixed1_11_4 { value: fixed_value }
    }

    pub fn to_f32(&self) -> f32 {
        self.value as f32 / (1 << Fixed1_11_4::FRACTIONAL_BITS) as f32
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_value = (value * (1 << Fixed1_11_4::FRACTIONAL_BITS) as f64) as i16;
        Fixed1_11_4 { value: fixed_value }
    }

    pub fn to_f64(&self) -> f64 {
        self.value as f64 / (1 << Fixed1_11_4::FRACTIONAL_BITS) as f64
    }

    pub fn get_int(&self) -> i16 {
        self.value >> Fixed1_11_4::FRACTIONAL_BITS
    }

    pub fn get_frac(&self) -> i16 {
        self.value & Fixed1_11_4::FRACTIONAL_MASK
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.value.to_le_bytes()
    }
}


impl Add for Fixed1_11_4 {
    type Output = Fixed1_11_4;
    
    fn add(self, rhs: Self) -> Self::Output {
        Fixed1_11_4 {
            value: self.value.wrapping_add(rhs.value)
        }
    }
}

impl Sub for Fixed1_11_4 {
    type Output = Fixed1_11_4;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Fixed1_11_4 {
            value: self.value.wrapping_sub(rhs.value)
        }
    }
}

impl Mul for Fixed1_11_4 {
    type Output = Fixed1_11_4;
    
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        Fixed1_11_4 {
            value: ((lhs_val * rhs_val) >> Fixed1_11_4::FRACTIONAL_BITS) as i16
        }
    }
}

impl Div for Fixed1_11_4 {
    type Output = Fixed1_11_4;
    
    fn div(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        if rhs_val == 0 {
            panic!("Division by zero in Fixed1_11_4");
        }

        Fixed1_11_4 {
            value: ((lhs_val << Fixed1_11_4::FRACTIONAL_BITS) / rhs_val) as i16
        }
    }
}

impl PartialEq for Fixed1_11_4 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Fixed1_11_4 {}

impl PartialOrd for Fixed1_11_4 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Fixed1_11_4 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Default for Fixed1_11_4 {
    fn default() -> Self {
        Fixed1_11_4 { value: 0 }
    }
}

impl From<i16> for Fixed1_11_4 {
    fn from(value: i16) -> Self {
        Fixed1_11_4::from_i16(value)
    }
}

impl Into<i16> for Fixed1_11_4 {
    fn into(self) -> i16 {
        self.to_i16()
    }
}

impl From<f32> for Fixed1_11_4 {
    fn from(value: f32) -> Self {
        Fixed1_11_4::from_f32(value)
    }
}

impl Into<f32> for Fixed1_11_4 {
    fn into(self) -> f32 {
        self.to_f32()
    }
}

impl From<f64> for Fixed1_11_4 {
    fn from(value: f64) -> Self {
        Fixed1_11_4::from_f64(value)
    }
}

impl Into<f64> for Fixed1_11_4 {
    fn into(self) -> f64 {
        self.to_f64()
    }
}

impl Debug for Fixed1_11_4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.value;
        let type_name = stringify!(Fixed1_11_4);

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
        assert!((a - b).abs() < EPSILON, "{} - Expected: {:.6}, Got: {:.6}", msg, b, a);
    }

    fn assert_f64_eq(a: f64, b: f64, msg: &str) {
        assert!((a - b).abs() < EPSILON_F64, "{} - Expected: {:.10}, Got: {:.10}", msg, b, a);
    }

    #[test]
    fn test_from_to_f32() {
        let val1 = Fixed1_11_4::from_f32(2.5);
        assert_f32_eq(val1.to_f32(), 2.5, "Positive value 2.5");

        let val2 = Fixed1_11_4::from_f32(-1.75);
        assert_f32_eq(val2.to_f32(), -1.75, "Negative value -1.75");

        let val3 = Fixed1_11_4::from_f32(0.0);
        assert_f32_eq(val3.to_f32(), 0.0, "Zero value");

        // Max positive integer part is (1 << 11) - 1 = 2047.
        // Max representable value is 2047 + 15/16 = 2047.9375
        // Test near max positive value
        let near_max_val_f32 = 2047.9; // Slightly less than 2047.9375
        let val4 = Fixed1_11_4::from_f32(near_max_val_f32);
        // Expected raw value: (2047.9 * 16.0) as i16 = 32766
        assert_f32_eq(val4.to_f32(), 32766_i16 as f32 / 16.0, "Near max positive value");

        // Min representable value is -2048.0
        let val5 = Fixed1_11_4::from_f32(-2048.0); // min value
        assert_f32_eq(val5.to_f32(), -2048.0, "Min negative value (-2048.0)");
        assert_eq!(val5.to_i16(), i16::MIN, "Min value should be i16::MIN internally");
    }

    #[test]
    fn test_from_to_f64() {
        let val1 = Fixed1_11_4::from_f64(123.45);
        assert_f64_eq(val1.to_f64(), ((123.45 * 16.0) as i16) as f64 / 16.0, "Positive value f64");

        let val2 = Fixed1_11_4::from_f64(-98.76);
        assert_f64_eq(val2.to_f64(), ((-98.76 * 16.0) as i16) as f64 / 16.0, "Negative value f64");
    }

    #[test]
    fn test_addition() {
        let a = Fixed1_11_4::from_f32(100.5);  // 100 + 8/16
        let b = Fixed1_11_4::from_f32(50.25); //  50 + 4/16
        assert_f32_eq((a + b).to_f32(), 150.75, "Addition"); // 150 + 12/16
    }

    #[test]
    fn test_subtraction() {
        let a = Fixed1_11_4::from_f32(150.75);
        let b = Fixed1_11_4::from_f32(50.25);
        assert_f32_eq((a - b).to_f32(), 100.5, "Subtraction");
    }

    #[test]
    fn test_multiplication() {
        let a = Fixed1_11_4::from_f32(10.0);
        let b = Fixed1_11_4::from_f32(2.5); // 2 + 8/16
        assert_f32_eq((a * b).to_f32(), 25.0, "Multiplication");

        let c = Fixed1_11_4::from_f32(-10.0);
        let d = Fixed1_11_4::from_f32(2.5);
        assert_f32_eq((c * d).to_f32(), -25.0, "Multiplication with negative");
    }

    #[test]
    fn test_division() {
        let a = Fixed1_11_4::from_f32(25.0);
        let b = Fixed1_11_4::from_f32(2.0);
        assert_f32_eq((a / b).to_f32(), 12.5, "Division"); // 12 + 8/16

        let c = Fixed1_11_4::from_f32(-25.0);
        let d = Fixed1_11_4::from_f32(2.0);
        assert_f32_eq((c / d).to_f32(), -12.5, "Division with negative");
    }

    #[test]
    #[should_panic(expected = "Division by zero in Fixed1_11_4")]
    fn test_division_by_zero() {
        let a = Fixed1_11_4::from_f32(1.0);
        let b = Fixed1_11_4::from_f32(0.0);
        let _ = a / b;
    }
    
    #[test]
    fn test_debug_format() {
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(0.0)), "Fixed1_11_4(0.0)");
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(1.0)), "Fixed1_11_4(1.0)");
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(-1.0)), "Fixed1_11_4(-1.0)");
        
        // 2.5 = 2 + 8/16
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(2.5)), "Fixed1_11_4(2.5)");
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(-2.5)), "Fixed1_11_4(-2.5)");
        
        // 0.125 = 2/16
        assert_eq!(format!("{:?}", Fixed1_11_4::from_f32(0.125)), "Fixed1_11_4(0.125)");
        
        // Smallest positive fraction: 1/16 = 0.0625
        let smallest_pos_frac = Fixed1_11_4::from_i16(1); // raw value 1
        assert_eq!(format!("{:?}", smallest_pos_frac), "Fixed1_11_4(0.0625)");

        // Test min value -2048.0 (i16::MIN internally)
        let min_val_fixed = Fixed1_11_4::from_i16(i16::MIN);
        assert_f32_eq(min_val_fixed.to_f32(), -2048.0, "Value check for min val (-2048.0)");
        assert_eq!(format!("{:?}", min_val_fixed), "Fixed1_11_4(-2048.0)");

        // Test max positive value 2047.9375 (i16::MAX internally)
        let max_pos_fixed = Fixed1_11_4::from_i16(i16::MAX); // raw value 32767
        assert_f32_eq(max_pos_fixed.to_f32(), 2047.9375, "Value check for max val (2047.9375)");
        assert_eq!(format!("{:?}", max_pos_fixed), "Fixed1_11_4(2047.9375)");

        // Test a value like -1024.0625 (-1024 - 1/16)
        let specific_neg_val = Fixed1_11_4::from_f32(-1024.0625);
        // Raw: (-1024.0625 * 16.0) as i16 = (-16384.0 - 1.0) as i16 = -16385
        assert_eq!(specific_neg_val.to_i16(), -16385, "Internal check for -1024.0625");
        assert_eq!(format!("{:?}", specific_neg_val), "Fixed1_11_4(-1024.0625)");
    }

    #[test]
    fn test_get_int_frac() {
        // 3.75 = 3 + 12/16. Raw value: (3.75 * 16) = 60
        let a = Fixed1_11_4::from_f32(3.75); 
        assert_eq!(a.get_int(), 3, "Integer part of 3.75");
        assert_eq!(a.get_frac(), 12, "Fractional part of 3.75 (12/16)");

        // -3.75. Raw value: (-3.75 * 16) = -60
        // get_int: -60 >> 4 = -4 (arithmetic shift)
        // get_frac: -60 & 0xF = 4. (-3.75 = -4 + 4/16)
        let b = Fixed1_11_4::from_f32(-3.75);
        assert_eq!(b.get_int(), -4, "Integer part of -3.75");
        assert_eq!(b.get_frac(), 4, "Fractional part of -3.75 (4/16)");

        // -2048.0. Raw value: i16::MIN = -32768
        let c = Fixed1_11_4::from_f32(-2048.0); 
        assert_eq!(c.get_int(), -2048, "Integer part of -2048.0");
        assert_eq!(c.get_frac(), 0, "Fractional part of -2048.0");
    }

    #[test]
    fn test_overflow_behavior() {
        // Max positive value is 2047.9375 (raw i16::MAX = 32767)
        let max_val = Fixed1_11_4::from_i16(i16::MAX); 
        // Smallest positive fraction is 0.0625 (raw 1)
        let smallest_frac = Fixed1_11_4::from_i16(1); 

        // Adding smallest fraction to max_val should overflow to negative
        // i16::MAX (32767) + 1 = i16::MIN (-32768)
        let overflow_add = max_val + smallest_frac;
        assert_eq!(overflow_add.to_i16(), i16::MIN, "Addition overflow to i16::MIN");
        assert_f32_eq(overflow_add.to_f32(), -2048.0, "Addition overflow check to -2048.0");

        // Multiplication overflow
        // Example: 100.0 * 30.0 = 3000.0 (out of range for Fixed1_11_4, max is ~2047)
        let val_100 = Fixed1_11_4::from_f32(100.0); // raw 1600
        let val_30 = Fixed1_11_4::from_f32(30.0);   // raw 480
        // (1600 * 480) >> 4 = 768000 >> 4 = 48000
        // 48000 as i16 wraps to 48000 - 65536 = -17536
        // -17536 / 16.0 = -1096.0
        let mul_overflow = val_100 * val_30;
        assert_f32_eq(mul_overflow.to_f32(), -1096.0, "Multiplication overflow check (100*30)");
    }
}
