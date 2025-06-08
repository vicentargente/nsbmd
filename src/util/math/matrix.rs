use std::{fmt::Debug, ops::Mul};

use crate::error::AppError;

#[derive(Clone)]
pub struct Matrix {
    width: u32,
    height: u32,
    data: Vec<f32>
}

impl Matrix {
    const SINGULARITY_THRESHOLD: f32 = 1e-6;

    pub fn new(width: u32, height: u32, data: Vec<f32>) -> Result<Matrix, AppError> {
        if (width as usize) * (height as usize) != data.len() {
            return Err(AppError::new("data size does not match width and height"));
        }

        Ok(Matrix {
            width,
            height,
            data
        })
    }

    pub fn from_bidimensional_array(data: Vec<Vec<f32>>) -> Result<Matrix, AppError> {
        let height = data.len();
        if height == 0 {
            return Ok(Matrix { width: 0, height: 0, data: Vec::new() });
        }

        let expected_width = data[0].len();

        let mut plain_data = Vec::with_capacity(expected_width * expected_width);
        
        for (i, row) in data.iter().enumerate() {
            if row.len() != expected_width {
                return Err(AppError::new(&format!("row number {} does not match the expected width. Expected: {}. Found: {}", i, expected_width, row.len())));
            }

            plain_data.extend_from_slice(row);
        }

        Self::new(expected_width as u32, height as u32, plain_data)
    }

    pub fn identity(n: u32) -> Matrix {
        let n_usize = n as usize;
        let mut data = vec![0.0; n_usize * n_usize];
        for cell in data.iter_mut().step_by(n_usize + 1) {
            *cell = 1.0;
        }

        Matrix {
            width: n,
            height: n,
            data
        }
    }

    pub fn zeros(width: u32, height: u32) -> Matrix {
        let data = vec![0.0; width as usize * height as usize];

        Matrix {
            width,
            height,
            data
        }
    }

    pub fn swap_rows(&mut self, row_1: u32, row_2: u32) -> Result<(), AppError> {
        if row_1 >= self.height {
            return Err(AppError::new(&format!("row_1 cannot exceeded height. Given: {}, Max allowed: {}", row_1, self.height - 1)));
        }

        if row_2 >= self.height {
            return Err(AppError::new(&format!("row_2 cannot exceeded height. Given: {}, Max allowed: {}", row_2, self.height - 1)));
        }

        let mut index_1 = self.get_index(row_1, 0);
        let mut index_2 = self.get_index(row_2, 0);
        for _ in 0..self.width {
            self.data.swap(index_1, index_2);

            index_1 += 1;
            index_2 += 1;
        }

        Ok(())
    }

    pub fn invert(&mut self) -> Result<(), AppError> {
        Ok(())
    }

    pub fn inverted(&self) -> Result<Matrix, AppError> {
        if self.width != self.height {
            return Err(AppError::new("Non square matrix cannot be inverted"));
        }

        let n = self.width;
        let mut inverted = Matrix::identity(n);
        let mut original = self.clone();

        for col_i in 0..self.width {
            // Get the maximum pivot row for the current column
            {
                let (max_row_index, max_value) = original.get_max_value_at_column_from_row(col_i, col_i)?;

                if max_value < Self::SINGULARITY_THRESHOLD {
                    return Err(AppError::new(&format!("Matrix is singular. Cannot be inverted. Column: {}, Max value: {}", col_i, max_value)));
                }
    
                if max_row_index != col_i {
                    original.swap_rows(col_i, max_row_index)?;
                    inverted.swap_rows(col_i, max_row_index)?;
                }
            }

            // Normalize the pivot row
            {
                let pivot_value = original.data[original.get_index(col_i, col_i)];
                
                for col_j in col_i..n {
                    let index = original.get_index(col_i, col_j);
                    original.data[index] /= pivot_value;
                }

                for col_j in 0..n {
                    let index = inverted.get_index(col_i, col_j);
                    inverted.data[index] /= pivot_value;
                }
            }

            // Eliminate the other rows
            for row_i in 0..n {
                if row_i == col_i {
                    continue;
                }

                let factor = original.data[original.get_index(row_i, col_i)];

                for col_j in col_i..n {
                    let index_l = original.get_index(row_i, col_j);
                    let index_r = original.get_index(col_i, col_j);
                    original.data[index_l] -= factor * original.data[index_r];
                }

                for col_j in 0..n {
                    let index_l = inverted.get_index(row_i, col_j);
                    let index_r = inverted.get_index(col_i, col_j);
                    inverted.data[index_l] -= factor * inverted.data[index_r];
                }
            }
        }
        
        Ok(inverted)
    }

    pub fn get(&self, row: u32, column: u32) -> Result<f32, AppError> {
        if row >= self.height {
            return Err(AppError::new(&format!("row exceeded height. Given: {}, Max allowed: {}", row, self.height - 1)));
        }

        if column >= self.width {
            return Err(AppError::new(&format!("column exceeded width. Given: {}, Max allowed: {}", column, self.width - 1)));
        }

        let index = self.get_index(row, column);
        Ok(self.data[index])
    }

    pub fn set(&mut self, row: u32, column: u32, value: f32) -> Result<(), AppError> {
        if row >= self.height {
            return Err(AppError::new(&format!("row exceeded height. Given: {}, Max allowed: {}", row, self.height - 1)));
        }

        if column >= self.width {
            return Err(AppError::new(&format!("column exceeded width. Given: {}, Max allowed: {}", column, self.width - 1)));
        }

        let index = self.get_index(row, column);
        self.data[index] = value;

        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row as usize) * (self.width as usize) + column as usize
    }

    fn get_max_value_at_column_from_row(&self, column: u32, from_row: u32) -> Result<(u32, f32), AppError> {
        if column >= self.width {
            return Err(AppError::new(&format!("column ({}) exceeded width ({}).", column, self.width - 1)));
        }

        if from_row >= self.height {
            return Err(AppError::new(&format!("from_row ({}) must be less than height ({}). Cannot find pivot in column {}.", from_row, self.height, column)));
        }

        let mut max_abs_value = self.data[self.get_index(from_row, column)].abs();
        let mut pivot_row_index = from_row;

        for current_row in (from_row + 1)..self.height {
            let current_abs_value = self.data[self.get_index(current_row, column)].abs();
            if current_abs_value > max_abs_value {
                max_abs_value = current_abs_value;
                pivot_row_index = current_row;
            }
        }

        Ok((pivot_row_index, max_abs_value))
    }

    pub fn can_be_multiplied(&self, other: &Matrix) -> bool {
        self.width == other.height
    }
}


impl Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Matrix");
        debug_struct.field("width", &self.width);
        debug_struct.field("height", &self.height);

        if self.width == 0 || self.height == 0 {
            // For an empty or invalid matrix, show data as an empty list.
            // An empty Vec will be formatted as `[]` by default.
            debug_struct.field("data", &Vec::<Vec<String>>::new());
        } else {
            let mut formatted_rows: Vec<Vec<String>> = Vec::with_capacity(self.height as usize);
            for r_idx in 0..self.height {
                let mut current_row_elements: Vec<String> = Vec::with_capacity(self.width as usize);
                for c_idx in 0..self.width {
                    let value = self.data[self.get_index(r_idx, c_idx)];
                    current_row_elements.push(format!("{:.6}", value));
                }
                formatted_rows.push(current_row_elements);
            }
            debug_struct.field("data", &formatted_rows);
        }
        
        debug_struct.finish()
    }
}

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        if !self.can_be_multiplied(&rhs) {
            panic!("Matrix multiplication requires the width of the first matrix to match the height of the second matrix.");
        }

        let mut result_data = vec![0.0; (self.height * rhs.width) as usize];
        for i in 0..self.height {
            for j in 0..rhs.width {
                let mut sum = 0.0;
                for k in 0..rhs.height {
                    sum += self.data[self.get_index(i, k)] * rhs.data[rhs.get_index(k, j)];
                }

                let result_index = (i * rhs.width + j) as usize;
                result_data[result_index] = sum;
            }
        }

        Matrix {
            width: rhs.width,
            height: self.height,
            data: result_data
        }
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn can_create_from_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let matrix = Matrix::new(3, 3, data).expect("Matrix did not initialize correctly");

        assert_eq!(matrix.width, 3);
        assert_eq!(matrix.height, 3);
        assert_eq!(matrix.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn can_create_from_bidimensional_array() {
        let data = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0]
        ];

        let matrix = Matrix::from_bidimensional_array(data).expect("Matrix did not initialize correctly");

        assert_eq!(matrix.width, 3);
        assert_eq!(matrix.height, 3);
        assert_eq!(matrix.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn can_create_identity() {
        let matrix = Matrix::identity(3);

        assert_eq!(matrix.width, 3);
        assert_eq!(matrix.height, 3);
        assert_eq!(matrix.data, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn can_create_zeros() {
        let matrix = Matrix::zeros(3, 3);

        assert_eq!(matrix.width, 3);
        assert_eq!(matrix.height, 3);
        assert_eq!(matrix.data, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn can_get_index() {
        let matrix = Matrix::zeros(3, 3);
        
        assert_eq!(matrix.get_index(0, 0), 0);
        assert_eq!(matrix.get_index(0, 1), 1);
        assert_eq!(matrix.get_index(1, 0), 3);
        assert_eq!(matrix.get_index(1, 2), 5);
        assert_eq!(matrix.get_index(2, 2), 8);
    }

    #[test]
    fn can_swap_rows() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut matrix = Matrix::new(3, 3, data).expect("Matrix did not initialize correctly");

        matrix.swap_rows(1, 2).expect("Could not swap rows");

        assert_eq!(matrix.data, vec![1.0, 2.0, 3.0, 7.0, 8.0, 9.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn can_get_max_value_at_column() {
        let data = vec![7.0, 2.0, 3.0, 4.0, 8.0, 6.0, 1.0, 5.0, 9.0];
        let matrix = Matrix::new(3, 3, data).expect("Matrix did not initialize correctly");

        let (max_row_index, max_value) = matrix.get_max_value_at_column_from_row(1, 0).expect("Could not get max value at column");
        assert_eq!(max_row_index, 1);
        assert_eq!(max_value, 8.0);

        let (max_row_index, max_value) = matrix.get_max_value_at_column_from_row(0, 0).expect("Could not get max value at column");
        assert_eq!(max_row_index, 0);
        assert_eq!(max_value, 7.0);

        let (max_row_index, max_value) = matrix.get_max_value_at_column_from_row(2, 0).expect("Could not get max value at column");
        assert_eq!(max_row_index, 2);
        assert_eq!(max_value, 9.0);
    }

    #[test]
    fn can_get_inverted_matrix() {
        let data = vec![0.0, 1.0, 2.0, 1.0, 3.0, 4.0, 4.0, 3.0, 2.0];
        let matrix = Matrix::new(3, 3, data).expect("Matrix did not initialize correctly");

        let inverted = matrix.inverted().expect("Matrix could not be inverted");

        assert_eq!(inverted.width, 3);
        assert_eq!(inverted.height, 3);
        
        let expected = vec![1.5, -1.0, 0.5, -3.5, 2.0, -0.5, 2.25, -1.0, 0.25];
        for (i, val) in inverted.data.iter().enumerate() {
            assert!((val - expected[i]).abs() < 1e-6, "Value at index {} does not match. Expected: {}, Found: {}", i, expected[i], val);
        }
    }

    #[test]
    fn cannot_invert_non_square_matrix() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let matrix = Matrix::new(2, 3, data).expect("Matrix did not initialize correctly");

        let result = matrix.inverted();
        assert!(result.is_err(), "Expected an error when inverting a non-square matrix");
    }

    #[test]
    fn cannot_invert_singular_matrix() {
        let data = vec![0.0, 2.0, 3.0, 0.0, 5.0, 6.0, 0.0, 8.0, 9.0];
        let matrix = Matrix::new(3, 3, data).expect("Matrix did not initialize correctly");

        let result = matrix.inverted();
        assert!(result.is_err(), "Expected an error when inverting a singular matrix");
    }

    #[test]
    fn can_multiply_matrices() {
        {
            let data_a = vec![1.0, 4.0, 7.0, 2.0, 5.0 ,8.0, 3.0, 6.0, 9.0];
            let matrix_a = Matrix::new(3, 3, data_a).expect("Matrix A did not initialize correctly");
    
            let data_b = vec![1.0, -1.0, 2.0, 2.0, -1.0, 2.0, 3.0, -3.0, 0.0];
            let matrix_b = Matrix::new(3, 3, data_b).expect("Matrix B did not initialize correctly");
    
            let result = matrix_a * matrix_b;
    
            assert_eq!(result.width, 3);
            assert_eq!(result.height, 3);
            assert_eq!(result.data, vec![30.0, -26.0, 10.0, 36.0, -31.0, 14.0, 42.0, -36.0, 18.0]);
        }

        {
            // Translation matrix test
            let data_a = vec![1.0, 0.0, 0.0, 0.0, 1.0, 4.0, 0.0, 0.0, 1.0];
            let matrix_a = Matrix::new(3, 3, data_a).expect("Matrix A did not initialize correctly");

            let data_b = vec![5.0, 2.0, 1.0];
            let matrix_b = Matrix::new(1, 3, data_b).expect("Matrix B did not initialize correctly");

            let result = matrix_a * matrix_b;

            assert_eq!(result.width, 1);
            assert_eq!(result.height, 3);
            assert_eq!(result.data, vec![5.0, 6.0, 1.0]);
        }        
    }

    #[test]
    fn test_can_multiply_matrices_scenarios() {
        // Case 1: Compatible A(2x3) * B(3x4) -> true
        // A has height 2, width 3. B has height 3, width 4.
        let matrix_a1 = Matrix::zeros(3, 2); 
        let matrix_b1 = Matrix::zeros(4, 3);
        assert!(matrix_a1.can_be_multiplied(&matrix_b1), "A(2x3) * B(3x4) should be compatible");

        // Case 2: Incompatible A(2x3) * B(2x4) -> false
        // A has height 2, width 3. B has height 2, width 4.
        let matrix_a2 = Matrix::zeros(3, 2);
        let matrix_b2 = Matrix::zeros(4, 2);
        assert!(!matrix_a2.can_be_multiplied(&matrix_b2), "A(2x3) * B(2x4) should be incompatible");

        // Case 3: Compatible square matrices A(3x3) * B(3x3) -> true
        let matrix_a3 = Matrix::zeros(3, 3);
        let matrix_b3 = Matrix::zeros(3, 3);
        assert!(matrix_a3.can_be_multiplied(&matrix_b3), "A(3x3) * B(3x3) should be compatible");

        // Case 4: Compatible row vector * column vector A(1x5) * B(5x1) -> true
        // A has height 1, width 5. B has height 5, width 1.
        let matrix_a4 = Matrix::zeros(5, 1); 
        let matrix_b4 = Matrix::zeros(1, 5);
        assert!(matrix_a4.can_be_multiplied(&matrix_b4), "A(1x5) * B(5x1) should be compatible");

        // Case 5: Compatible column vector * row vector A(5x1) * B(1x5) -> true
        // A has height 5, width 1. B has height 1, width 5.
        let matrix_a5 = Matrix::zeros(1, 5);
        let matrix_b5 = Matrix::zeros(5, 1);
        assert!(matrix_a5.can_be_multiplied(&matrix_b5), "A(5x1) * B(1x5) should be compatible");

        // Case 6: Incompatible A(2x2) * B(3x1) -> false
        // A has height 2, width 2. B has height 3, width 1.
        let matrix_a6 = Matrix::zeros(2, 2);
        let matrix_b6 = Matrix::zeros(1, 3);
        assert!(!matrix_a6.can_be_multiplied(&matrix_b6), "A(2x2) * B(3x1) should be incompatible");
    }

    #[test]
    #[should_panic(expected = "Matrix multiplication requires the width of the first matrix to match the height of the second matrix.")]
    fn cannot_multiply_incompatible_matrices() {
        let data_a = vec![1.0, 2.0, 3.0, 4.0];
        let matrix_a = Matrix::new(2, 2, data_a).expect("Matrix A did not initialize correctly");

        let data_b = vec![1.0, 2.0, 3.0];
        let matrix_b = Matrix::new(3, 1, data_b).expect("Matrix B did not initialize correctly");

        assert!(!matrix_a.can_be_multiplied(&matrix_b), "Expected matrices to be incompatible for multiplication");

        let _ = matrix_a * matrix_b;
    }
}
