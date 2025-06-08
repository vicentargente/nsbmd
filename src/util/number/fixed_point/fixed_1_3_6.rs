use std::{fmt::Debug, ops::{Add, Div, Mul, Sub}};

#[derive(Clone, Copy)]
pub struct Fixed1_3_6 {
    value: i16
}

impl Fixed1_3_6 {
    const _INTEGER_BITS: usize = 3;
    const FRACTIONAL_BITS: usize = 6;

    const FRACTIONAL_MASK: i16 = (1 << Self::FRACTIONAL_BITS) - 1;
    const NUMBER_DATA_MASK: i16 = (1 << (Self::FRACTIONAL_BITS + 1)) - 1;
    const VOID_DATA_MASK: i16 = !Self::NUMBER_DATA_MASK;
    const SIGN_MASK: i16 = 1 << (Self::FRACTIONAL_BITS + 1);

    pub fn from_i16(value: i16) -> Self {
        let masked = value & Fixed1_3_6::NUMBER_DATA_MASK;
        let value = if masked & Fixed1_3_6::SIGN_MASK != 0 {
            masked | Fixed1_3_6::VOID_DATA_MASK
        }
        else {
            masked
        };

        Fixed1_3_6 { value }
    }

    pub fn to_i16(&self) -> i16 {
        self.value
    }

    pub fn from_f32(value: f32) -> Self {
        let max = 1.0 - 1.0 / (1 << Self::FRACTIONAL_BITS) as f32;
        let clamped = value.clamp(-1.0, max);
        let fixed_value = (clamped * (1 << Self::FRACTIONAL_BITS) as f32) as i16;
        Fixed1_3_6 { value: fixed_value }
    }

    pub fn to_f32(&self) -> f32 {
        self.value as f32 / (1 << Fixed1_3_6::FRACTIONAL_BITS) as f32
    }

    pub fn from_f64(value: f64) -> Self {
        let max = 1.0 - 1.0 / (1 << Self::FRACTIONAL_BITS) as f64;
        let clamped = value.clamp(-1.0, max);
        let fixed_value = (clamped * (1 << Self::FRACTIONAL_BITS) as f64) as i16;
        Fixed1_3_6 { value: fixed_value }
    }

    pub fn to_f64(&self) -> f64 {
        self.value as f64 / (1 << Fixed1_3_6::FRACTIONAL_BITS) as f64
    }

    pub fn get_int(&self) -> i16 {
        self.value >> Fixed1_3_6::FRACTIONAL_BITS
    }

    pub fn get_frac(&self) -> i16 {
        self.value & Fixed1_3_6::FRACTIONAL_MASK
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.value.to_le_bytes()
    }

}

impl Add for Fixed1_3_6 {
    type Output = Fixed1_3_6;
    
    fn add(self, rhs: Self) -> Self::Output {
        Fixed1_3_6 {
            value: self.value + rhs.value
        }
    }
}

impl Sub for Fixed1_3_6 {
    type Output = Fixed1_3_6;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Fixed1_3_6 {
            value: self.value - rhs.value
        }
    }
}

impl Mul for Fixed1_3_6 {
    type Output = Fixed1_3_6;
    
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        Fixed1_3_6 {
            value: ((lhs_val * rhs_val) >> Fixed1_3_6::FRACTIONAL_BITS) as i16
        }
    }
}

impl Div for Fixed1_3_6 {
    type Output = Fixed1_3_6;
    
    fn div(self, rhs: Self) -> Self::Output {
        let lhs_val = self.value as i32;
        let rhs_val = rhs.value as i32;

        if rhs_val == 0 {
            panic!("Division by zero in Fixed1_3_6");
        }

        Fixed1_3_6 {
            value: ((lhs_val << Fixed1_3_6::FRACTIONAL_BITS) / rhs_val) as i16
        }
    }
}

impl PartialEq for Fixed1_3_6 {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Fixed1_3_6 {}

impl PartialOrd for Fixed1_3_6 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Fixed1_3_6 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Default for Fixed1_3_6 {
    fn default() -> Self {
        Fixed1_3_6 { value: 0 }
    }
}

impl From<i16> for Fixed1_3_6 {
    fn from(value: i16) -> Self {
        Fixed1_3_6::from_i16(value)
    }
}

impl Into<i16> for Fixed1_3_6 {
    fn into(self) -> i16 {
        self.to_i16()
    }
}

impl From<f32> for Fixed1_3_6 {
    fn from(value: f32) -> Self {
        Fixed1_3_6::from_f32(value)
    }
}

impl Into<f32> for Fixed1_3_6 {
    fn into(self) -> f32 {
        self.to_f32()
    }
}

impl From<f64> for Fixed1_3_6 {
    fn from(value: f64) -> Self {
        Fixed1_3_6::from_f64(value)
    }
}

impl Into<f64> for Fixed1_3_6 {
    fn into(self) -> f64 {
        self.to_f64()
    }
}

impl Debug for Fixed1_3_6 {
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

        write!(f, "Fixed1_3_6({}{}.{})", sign, integer, fractional_str)
    }
}
