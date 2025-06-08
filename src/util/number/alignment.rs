pub fn get_4_byte_alignment(a: usize) -> usize {
    (a.wrapping_sub(1) & !3).wrapping_add(4)
}

pub fn get_16_byte_alignment(a: usize) -> usize {
    (a.wrapping_sub(1) & !15).wrapping_add(16)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_4_byte_alignment() {
        assert_eq!(get_4_byte_alignment(0), 0);
        assert_eq!(get_4_byte_alignment(1), 4);
        assert_eq!(get_4_byte_alignment(2), 4);
        assert_eq!(get_4_byte_alignment(3), 4);
        assert_eq!(get_4_byte_alignment(4), 4);
        assert_eq!(get_4_byte_alignment(5), 8);
        assert_eq!(get_4_byte_alignment(6), 8);
        assert_eq!(get_4_byte_alignment(7), 8);
        assert_eq!(get_4_byte_alignment(8), 8);
    }

    #[test]
    fn test_get_16_byte_alignment() {
        assert_eq!(get_16_byte_alignment(0), 0);
        assert_eq!(get_16_byte_alignment(1), 16);
        assert_eq!(get_16_byte_alignment(2), 16);
        assert_eq!(get_16_byte_alignment(3), 16);
        assert_eq!(get_16_byte_alignment(4), 16);
        assert_eq!(get_16_byte_alignment(5), 16);
        assert_eq!(get_16_byte_alignment(15), 16);
        assert_eq!(get_16_byte_alignment(16), 16);
        assert_eq!(get_16_byte_alignment(17), 32);
    }
}