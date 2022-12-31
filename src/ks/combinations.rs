// Copyright 2022 by Daniel Winkelman. All rights reserved.

pub fn get_combinations(num_cells: usize, sum: usize) -> Vec<u64> {
    fn recurse(
        num_cells: usize,
        sum: usize,
        current_value: usize,
        accum: u64,
        output: &mut Vec<u64>,
    ) {
        if num_cells == 1 {
            assert!(current_value <= sum);
            output.push(accum | (1 << sum));
        } else {
            let lower_limit = {
                let forced_max =
                    (9 - (num_cells - 1)) * (num_cells - 1) + num_cells * (num_cells - 1) / 2;
                if forced_max < sum {
                    (sum - forced_max).min(9)
                } else {
                    1
                }
            };
            let upper_limit = {
                let forced_min =
                    (current_value - 1) * (num_cells - 1) + num_cells * (num_cells - 1) / 2;
                assert!(forced_min < sum);
                let ceiling = (sum - num_cells * (num_cells - 1) / 2) / num_cells;
                (sum - forced_min).min(ceiling).min(9)
            };
            for i in current_value.max(lower_limit)..=upper_limit {
                recurse(num_cells - 1, sum - i, i + 1, accum | (1 << i), output);
            }
        }
    }
    let mut output = vec![];
    recurse(num_cells, sum, 1, 0, &mut output);
    output
}

pub fn get_combinations_union(num_cells: usize, sum: usize) -> u64 {
    get_combinations(num_cells, sum)
        .into_iter()
        .fold(0, |accum, x| accum | x)
}

pub struct PossibleValues {
    bitmask: u64,
    index: usize,
}

impl PossibleValues {
    pub fn new(bitmask: u64) -> Self {
        Self { bitmask, index: 0 }
    }
}

impl Iterator for PossibleValues {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitmask == 0 {
            None
        } else {
            for i in self.index..64 {
                if self.bitmask & 1 == 1 {
                    self.bitmask >>= 1;
                    self.index = i + 1;
                    return Some(i);
                } else {
                    self.bitmask >>= 1;
                }
            }
            panic!("Should be unreachable");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ks::combinations::PossibleValues;

    use super::{get_combinations, get_combinations_union};

    #[test]
    fn test_single_cell() {
        let c = get_combinations(1, 5);
        assert_eq!(c.len(), 1);
        assert_eq!(PossibleValues::new(c[0]).collect::<Vec<usize>>(), vec![5]);
    }

    #[test]
    fn test_double_cell_odd() {
        let c = get_combinations(2, 13);
        assert_eq!(c.len(), 3);
        assert_eq!(
            PossibleValues::new(c[0]).collect::<Vec<usize>>(),
            vec![4, 9]
        );
        assert_eq!(
            PossibleValues::new(c[1]).collect::<Vec<usize>>(),
            vec![5, 8]
        );
        assert_eq!(
            PossibleValues::new(c[2]).collect::<Vec<usize>>(),
            vec![6, 7]
        );
    }

    #[test]
    fn test_double_cell_even() {
        let c = get_combinations(2, 14);
        assert_eq!(c.len(), 2);
        assert_eq!(
            PossibleValues::new(c[0]).collect::<Vec<usize>>(),
            vec![5, 9]
        );
        assert_eq!(
            PossibleValues::new(c[1]).collect::<Vec<usize>>(),
            vec![6, 8]
        );
    }

    #[test]
    fn test_multiple_cells() {
        /* https://en.wikipedia.org/wiki/Killer_sudoku#Cage_total_tables */
        assert_eq!(get_combinations(3, 15).len(), 8);
        assert_eq!(get_combinations(4, 15).len(), 6);
        assert_eq!(get_combinations(5, 15).len(), 1);
        assert_eq!(get_combinations(5, 25).len(), 12);
        assert_eq!(get_combinations(6, 25).len(), 4);
        assert_eq!(get_combinations(7, 33).len(), 3);
        assert_eq!(get_combinations(8, 40).len(), 1);
        assert_eq!(get_combinations(9, 45).len(), 1);
    }

    #[test]
    fn test_union() {
        assert_eq!(
            PossibleValues::new(get_combinations_union(2, 13)).collect::<Vec<usize>>(),
            vec![4, 5, 6, 7, 8, 9]
        );
    }
}
