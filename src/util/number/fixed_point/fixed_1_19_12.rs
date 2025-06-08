use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

#[derive(Clone, Copy)]
pub struct Fixed1_19_12 {
    value: i32
}

impl Fixed1_19_12 {
    const _INTEGER_BITS: usize = 19;
    const FRACTIONAL_BITS: usize = 12;
    const FRACTIONAL_MASK: i32 = (1 << Self::FRACTIONAL_BITS) - 1;

    pub fn from_i32(value: i32) -> Self {
        Fixed1_19_12 { value }
    }

    pub fn to_i32(&self) -> i32 {
        self.value
    }

    pub fn from_f32(value: f32) -> Self {
        let fixed_value = (value * (1 << Fixed1_19_12::FRACTIONAL_BITS) as f32) as i32;
        Fixed1_19_12 { value: fixed_value }
    }

    pub fn to_f32(&self) -> f32 {
        self.value as f32 / (1 << Fixed1_19_12::FRACTIONAL_BITS) as f32
    }

    pub fn from_f64(value: f64) -> Self {
        let fixed_value = (value * (1 << Fixed1_19_12::FRACTIONAL_BITS) as f64) as i32;
        Fixed1_19_12 { value: fixed_value }
    }

    pub fn to_f64(&self) -> f64 {
        self.value as f64 / (1 << Fixed1_19_12::FRACTIONAL_BITS) as f64
    }

    pub fn get_int(&self) -> i32 {
        self.value >> Fixed1_19_12::FRACTIONAL_BITS
    }

    pub fn get_frac(&self) -> i32 {
        self.value & Fixed1_19_12::FRACTIONAL_MASK
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.value.to_le_bytes()
    }

}

impl Add for Fixed1_19_12 {
    type Output = Fixed1_19_12;
    
    fn add(self, rhs: Self) -> Self::Output {
        Fixed1_19_12 {
            value: self.value.wrapping_add(rhs.value)
        }
    }
}

impl Sub for Fixed1_19_12 {
    type Output = Fixed1_19_12;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Fixed1_19_12 {
            value: self.value.wrapping_sub(rhs.value)
        }
    }
}

impl Mul for Fixed1_19_12 {
    type Output = Fixed1_19_12;
    
    fn mul(self, rhs: Self) -> Self::Output {
        // Use i64 for the intermediate product to prevent overflow
        let temp_product = (self.value as i64) * (rhs.value as i64);
        Fixed1_19_12 {
            value: (temp_product >> Self::FRACTIONAL_BITS) as i32
        }
    }
}

impl Div for Fixed1_19_12 {
    type Output = Fixed1_19_12;
    
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero in Fixed1_19_12");
        }
        // Use i64 for the dividend to prevent overflow before division
        let dividend = (self.value as i64) << Self::FRACTIONAL_BITS;
        Fixed1_19_12 {
            // The divisor rhs.value is i32, promoting to i64 for division
            value: (dividend / (rhs.value as i64)) as i32
        }
    }
}

impl PartialEq for Fixed1_19_12 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Fixed1_19_12 {}

impl PartialOrd for Fixed1_19_12 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Fixed1_19_12 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Default for Fixed1_19_12 {
    fn default() -> Self {
        Fixed1_19_12 { value: 0 }
    }
}

impl From<i32> for Fixed1_19_12 {
    fn from(value: i32) -> Self {
        Fixed1_19_12::from_i32(value)
    }
}

impl Into<i32> for Fixed1_19_12 {
    fn into(self) -> i32 {
        self.to_i32()
    }
}

impl From<f32> for Fixed1_19_12 {
    fn from(value: f32) -> Self {
        Fixed1_19_12::from_f32(value)
    }
}

impl Into<f32> for Fixed1_19_12 {
    fn into(self) -> f32 {
        self.to_f32()
    }
}

impl From<f64> for Fixed1_19_12 {
    fn from(value: f64) -> Self {
        Fixed1_19_12::from_f64(value)
    }
}

impl Into<f64> for Fixed1_19_12 {
    fn into(self) -> f64 {
        self.to_f64()
    }
}

impl Debug for Fixed1_19_12 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.value;
        let type_name = stringify!(Fixed1_19_12);

        let sign_str = if val < 0 { "-" } else { "" };

        let display_integer: i32;
        let fractional_numerator: u32;

        if val == i32::MIN {
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

    // Using a slightly tighter epsilon for f32 given more fractional bits
    const EPSILON_F32: f32 = 0.0001; 
    const EPSILON_F64: f64 = 0.000000001; // Epsilon for f64

    fn assert_f32_eq(a: f32, b: f32, msg: &str) {
        assert!((a - b).abs() < EPSILON_F32, "{} - Expected: {:.7}, Got: {:.7}", msg, b, a);
    }

    fn assert_f64_eq(a: f64, b: f64, msg: &str) {
        assert!((a - b).abs() < EPSILON_F64, "{} - Expected: {:.12}, Got: {:.12}", msg, b, a);
    }

    #[test]
    fn test_from_to_f32() {
        let val1 = Fixed1_19_12::from_f32(12345.678);
        assert_f32_eq(val1.to_f32(), ((12345.678 * 4096.0) as i32) as f32 / 4096.0, "Positive value");

        let val2 = Fixed1_19_12::from_f32(-9876.543);
        assert_f32_eq(val2.to_f32(), ((-9876.543 * 4096.0) as i32) as f32 / 4096.0, "Negative value");

        let val3 = Fixed1_19_12::from_f32(0.0);
        assert_f32_eq(val3.to_f32(), 0.0, "Zero value");

        // Max positive integer part is (1 << 19) - 1 = 524287.
        // Max representable value is 524287 + 4095/4096 = 524287.999755859375
        let near_max_val_f32 = 524287.9; 
        let val4 = Fixed1_19_12::from_f32(near_max_val_f32);
        assert_f32_eq(val4.to_f32(), ((near_max_val_f32 * 4096.0) as i32) as f32 / 4096.0, "Near max positive value");
        
        let max_fixed_val = Fixed1_19_12::from_i32(i32::MAX);
        assert_f32_eq(max_fixed_val.to_f32(), 524287.999755859375, "Max positive value (i32::MAX)");


        // Min representable value is -524288.0
        let min_fixed_val = Fixed1_19_12::from_i32(i32::MIN);
        assert_f32_eq(min_fixed_val.to_f32(), -524288.0, "Min negative value (i32::MIN)");
        assert_eq!(min_fixed_val.to_i32(), i32::MIN, "Min value should be i32::MIN internally");
    }

    #[test]
    fn test_from_to_f64() {
        let val1 = Fixed1_19_12::from_f64(123456.789);
        assert_f64_eq(val1.to_f64(), ((123456.789 * 4096.0) as i32) as f64 / 4096.0, "Positive value f64");

        let val2 = Fixed1_19_12::from_f64(-98765.4321);
        assert_f64_eq(val2.to_f64(), ((-98765.4321 * 4096.0) as i32) as f64 / 4096.0, "Negative value f64");
    }

    #[test]
    fn test_addition() {
        let a = Fixed1_19_12::from_f32(10000.5);  // 10000 + 2048/4096
        let b = Fixed1_19_12::from_f32(5000.25); //  5000 + 1024/4096
        assert_f32_eq((a + b).to_f32(), 15000.75, "Addition"); // 15000 + 3072/4096
    }

    #[test]
    fn test_subtraction() {
        let a = Fixed1_19_12::from_f32(15000.75);
        let b = Fixed1_19_12::from_f32(5000.25);
        assert_f32_eq((a - b).to_f32(), 10000.5, "Subtraction");
    }

    #[test]
    fn test_multiplication() {
        let a = Fixed1_19_12::from_f32(100.0);
        let b = Fixed1_19_12::from_f32(20.5); // 20 + 2048/4096
        assert_f32_eq((a * b).to_f32(), 2050.0, "Multiplication");

        let c = Fixed1_19_12::from_f32(-100.0);
        let d = Fixed1_19_12::from_f32(20.5);
        assert_f32_eq((c * d).to_f32(), -2050.0, "Multiplication with negative");
    }

    #[test]
    fn test_division() {
        let a = Fixed1_19_12::from_f32(2050.0);
        let b = Fixed1_19_12::from_f32(20.0);
        assert_f32_eq((a / b).to_f32(), 102.5, "Division"); 

        let c = Fixed1_19_12::from_f32(-2050.0);
        let d = Fixed1_19_12::from_f32(20.0);
        assert_f32_eq((c / d).to_f32(), -102.5, "Division with negative");
    }

    #[test]
    #[should_panic(expected = "Division by zero in Fixed1_19_12")]
    fn test_division_by_zero() {
        let a = Fixed1_19_12::from_f32(1.0);
        let b = Fixed1_19_12::from_f32(0.0);
        let _ = a / b;
    }
    
    #[test]
    fn test_debug_format() {
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(0.0)), "Fixed1_19_12(0.0)");
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(1.0)), "Fixed1_19_12(1.0)");
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(-1.0)), "Fixed1_19_12(-1.0)");
        
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(2.5)), "Fixed1_19_12(2.5)");
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(-2.5)), "Fixed1_19_12(-2.5)");
        
        // 0.125 = 512/4096
        assert_eq!(format!("{:?}", Fixed1_19_12::from_f32(0.125)), "Fixed1_19_12(0.125)");
        
        // Smallest positive fraction: 1/4096 = 0.000244140625
        let smallest_pos_frac = Fixed1_19_12::from_i32(1); // raw value 1
        assert_eq!(format!("{:?}", smallest_pos_frac), "Fixed1_19_12(0.000244140625)");

        let min_val_fixed = Fixed1_19_12::from_i32(i32::MIN);
        assert_f32_eq(min_val_fixed.to_f32(), -524288.0, "Value check for min val (-524288.0)");
        assert_eq!(format!("{:?}", min_val_fixed), "Fixed1_19_12(-524288.0)");

        let max_pos_fixed = Fixed1_19_12::from_i32(i32::MAX); 
        assert_f32_eq(max_pos_fixed.to_f32(), 524287.999755859375, "Value check for max val");
        assert_eq!(format!("{:?}", max_pos_fixed), "Fixed1_19_12(524287.999755859375)");

        let specific_neg_val = Fixed1_19_12::from_f32(-12345.678);
        // Raw: (-12345.678 * 4096.0) as i32 = -50567897
        // Debug output will be based on this raw value.
        // -50567897 / 4096.0 = -12345.677734375
        assert_eq!(format!("{:?}", specific_neg_val), "Fixed1_19_12(-12345.677734375)");
    }

    #[test]
    fn test_get_int_frac() {
        // 12345.678. Raw value: (12345.678 * 4096.0) as i32 = 50561753
        let a = Fixed1_19_12::from_f32(12345.678); 
        assert_eq!(a.get_int(), 12345, "Integer part of 12345.678");
        assert_eq!(a.get_frac(), 2776, "Fractional part of 12345.678 (2777/4096)"); // 0.678 * 4096 = 2776.928

        // -2.5. Raw value: (-2.5 * 4096.0) as i32 = -10240
        let b = Fixed1_19_12::from_f32(-2.5);
        assert_eq!(b.get_int(), -3, "Integer part of -2.5"); // floor(-2.5) if shift behaves like floor
        assert_eq!(b.get_frac(), 2048, "Fractional part of -2.5 (2048/4096)"); // -2.5 = -3 + 0.5

        // -524288.0. Raw value: i32::MIN = -2147483648
        let c = Fixed1_19_12::from_i32(i32::MIN); 
        assert_eq!(c.get_int(), -524288, "Integer part of -524288.0");
        assert_eq!(c.get_frac(), 0, "Fractional part of -524288.0");
    }

    #[test]
    fn test_overflow_behavior() {
        let max_val = Fixed1_19_12::from_i32(i32::MAX); 
        let smallest_frac = Fixed1_19_12::from_i32(1); 

        let overflow_add = max_val + smallest_frac; // i32::MAX + 1 = i32::MIN
        assert_eq!(overflow_add.to_i32(), i32::MIN, "Addition overflow to i32::MIN");
        assert_f32_eq(overflow_add.to_f32(), -524288.0, "Addition overflow check to -524288.0");

        // Multiplication overflow
        // Example: 200000.0 * 3.0 = 600000.0 (out of range for Fixed1_19_12, max int is ~524287)
        let val_200k = Fixed1_19_12::from_f32(200000.0); // raw 200000 * 4096 = 819200000
        let val_3 = Fixed1_19_12::from_f32(3.0);       // raw 3 * 4096 = 12288
        // Intermediate product: (819200000_i64 * 12288_i64) = 10066329600000_i64
        // Shifted: 10066329600000_i64 >> 12 = 2457600000_i64
        // Cast to i32: 2457600000_i64 as i32 wraps.
        // 2457600000 is 0x927C0000. As i32, this is -1837027328.
        // Resulting f32: -1837027328 / 4096.0 = -448493.0
        let mul_overflow = val_200k * val_3;
        assert_f32_eq(mul_overflow.to_f32(), -448576.0000000, "Multiplication overflow check (200k*3)");
    }
}
