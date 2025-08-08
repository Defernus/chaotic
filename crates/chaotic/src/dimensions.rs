use std::borrow::Cow;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Dimensions {
    dimensions: Cow<'static, [usize]>,
}

impl Dimensions {
    pub const fn new_static(dimensions: &'static [usize]) -> Self {
        Dimensions {
            dimensions: Cow::Borrowed(dimensions),
        }
    }

    pub fn new(dimensions: Vec<usize>) -> Self {
        Dimensions {
            dimensions: Cow::Owned(dimensions),
        }
    }

    /// Number of dimensions
    pub fn len(&self) -> usize {
        self.dimensions.len()
    }

    /// ND volume of the dimensions
    pub fn volume(&self) -> usize {
        self.dimensions.iter().product()
    }

    pub fn iter(&self) -> DimensionsIterator {
        let coordinates = vec![0; self.dimensions.len()];

        DimensionsIterator {
            dimensions: self.clone(),
            coordinates,
            started: false,
        }
    }

    pub fn index_to_pos(&self, index: usize) -> Vec<usize> {
        let mut pos = Vec::with_capacity(self.dimensions.len());
        let mut remaining = index;

        for &dim_size in self.dimensions.iter() {
            pos.push(remaining % dim_size);
            remaining /= dim_size;
        }

        pos
    }

    pub fn pos_to_index(&self, pos: &[usize]) -> usize {
        let mut index = 0;
        let mut multiplier = 1;

        for (i, &coord) in pos.iter().enumerate() {
            index += coord * multiplier;
            multiplier *= self.dimensions[i];
        }

        index
    }
}

impl Index<usize> for Dimensions {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.dimensions[index]
    }
}

#[derive(Debug, Clone)]
pub struct DimensionsIterator {
    dimensions: Dimensions,
    coordinates: Vec<usize>,
    started: bool,
}

impl Iterator for DimensionsIterator {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coordinates.is_empty() {
            return None;
        }

        // Return the current position first
        if !self.started {
            self.started = true;
            return Some(self.coordinates.clone());
        }

        // Increment coordinates in row-major order (first dimension varies fastest)
        for i in 0..self.dimensions.dimensions.len() {
            if self.coordinates[i] < self.dimensions.dimensions[i] - 1 {
                self.coordinates[i] += 1;
                return Some(self.coordinates.clone());
            } else {
                self.coordinates[i] = 0;
            }
        }

        // If we get here, we've exhausted all possibilities
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_pos_to_index() {
        // Test 1D case
        let dims_1d = Dimensions::new(vec![5]);
        assert_eq!(dims_1d.pos_to_index(&[0]), 0);
        assert_eq!(dims_1d.pos_to_index(&[1]), 1);
        assert_eq!(dims_1d.pos_to_index(&[4]), 4);

        // Test 2D case
        let dims_2d = Dimensions::new(vec![3, 4]);
        assert_eq!(dims_2d.pos_to_index(&[0, 0]), 0);
        assert_eq!(dims_2d.pos_to_index(&[1, 0]), 1);
        assert_eq!(dims_2d.pos_to_index(&[2, 0]), 2);
        assert_eq!(dims_2d.pos_to_index(&[0, 1]), 3);
        assert_eq!(dims_2d.pos_to_index(&[1, 1]), 4);
        assert_eq!(dims_2d.pos_to_index(&[2, 1]), 5);
        assert_eq!(dims_2d.pos_to_index(&[0, 2]), 6);
        assert_eq!(dims_2d.pos_to_index(&[2, 3]), 11);

        // Test 3D case
        let dims_3d = Dimensions::new(vec![2, 3, 4]);
        assert_eq!(dims_3d.pos_to_index(&[0, 0, 0]), 0);
        assert_eq!(dims_3d.pos_to_index(&[1, 0, 0]), 1);
        assert_eq!(dims_3d.pos_to_index(&[0, 1, 0]), 2);
        assert_eq!(dims_3d.pos_to_index(&[1, 1, 0]), 3);
        assert_eq!(dims_3d.pos_to_index(&[0, 2, 0]), 4);
        assert_eq!(dims_3d.pos_to_index(&[1, 2, 0]), 5);
        assert_eq!(dims_3d.pos_to_index(&[0, 0, 1]), 6);
        assert_eq!(dims_3d.pos_to_index(&[1, 2, 3]), 23);
    }

    #[test]
    fn test_dimension_index_to_pos() {
        // Test 1D case
        let dims_1d = Dimensions::new(vec![5]);
        assert_eq!(dims_1d.index_to_pos(0), vec![0]);
        assert_eq!(dims_1d.index_to_pos(1), vec![1]);
        assert_eq!(dims_1d.index_to_pos(4), vec![4]);

        // Test 2D case
        let dims_2d = Dimensions::new(vec![3, 4]);
        assert_eq!(dims_2d.index_to_pos(0), vec![0, 0]);
        assert_eq!(dims_2d.index_to_pos(1), vec![1, 0]);
        assert_eq!(dims_2d.index_to_pos(2), vec![2, 0]);
        assert_eq!(dims_2d.index_to_pos(3), vec![0, 1]);
        assert_eq!(dims_2d.index_to_pos(4), vec![1, 1]);
        assert_eq!(dims_2d.index_to_pos(5), vec![2, 1]);
        assert_eq!(dims_2d.index_to_pos(6), vec![0, 2]);
        assert_eq!(dims_2d.index_to_pos(11), vec![2, 3]);

        // Test 3D case
        let dims_3d = Dimensions::new(vec![2, 3, 4]);
        assert_eq!(dims_3d.index_to_pos(0), vec![0, 0, 0]);
        assert_eq!(dims_3d.index_to_pos(1), vec![1, 0, 0]);
        assert_eq!(dims_3d.index_to_pos(2), vec![0, 1, 0]);
        assert_eq!(dims_3d.index_to_pos(3), vec![1, 1, 0]);
        assert_eq!(dims_3d.index_to_pos(4), vec![0, 2, 0]);
        assert_eq!(dims_3d.index_to_pos(5), vec![1, 2, 0]);
        assert_eq!(dims_3d.index_to_pos(6), vec![0, 0, 1]);
        assert_eq!(dims_3d.index_to_pos(23), vec![1, 2, 3]);
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test that index_to_pos and pos_to_index are inverse operations
        let dims = Dimensions::new(vec![3, 4, 5]);

        // Test all indices in the valid range
        for index in 0..dims.volume() {
            let pos = dims.index_to_pos(index);
            let converted_back = dims.pos_to_index(&pos);
            assert_eq!(
                index, converted_back,
                "Roundtrip failed for index {}",
                index
            );
        }

        // Test some specific positions
        let test_positions = vec![
            vec![0, 0, 0],
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![0, 1, 2],
            vec![2, 0, 4],
        ];

        for pos in test_positions {
            let index = dims.pos_to_index(&pos);
            let converted_back = dims.index_to_pos(index);
            assert_eq!(
                pos, converted_back,
                "Roundtrip failed for position {:?}",
                pos
            );
        }
    }

    #[test]
    fn test_static_dimensions() {
        // Test with static dimensions
        static DIMS: &[usize] = &[2, 3];
        let dims = Dimensions::new_static(DIMS);

        assert_eq!(dims.pos_to_index(&[0, 0]), 0);
        assert_eq!(dims.pos_to_index(&[1, 2]), 5);
        assert_eq!(dims.index_to_pos(0), vec![0, 0]);
        assert_eq!(dims.index_to_pos(5), vec![1, 2]);
    }

    #[test]
    fn test_volume_calculation() {
        let dims_1d = Dimensions::new(vec![5]);
        assert_eq!(dims_1d.volume(), 5);

        let dims_2d = Dimensions::new(vec![3, 4]);
        assert_eq!(dims_2d.volume(), 12);

        let dims_3d = Dimensions::new(vec![2, 3, 4]);
        assert_eq!(dims_3d.volume(), 24);
    }

    #[test]
    fn test_iter_dimensions() {
        let dims = Dimensions::new(vec![2, 3, 4]);
        for (index, pos) in dims.iter().enumerate() {
            let expected_pos = dims.index_to_pos(index);
            assert_eq!(pos, expected_pos, "Mismatch at index {}", index);
        }
    }
}
