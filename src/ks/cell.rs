// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::util::onehot;
use std::fmt::Display;

#[derive(Clone, Copy)]
pub struct Cell {
    possible_values: u64,
}

impl Cell {
    pub fn get_solution(&self) -> Option<usize> {
        onehot(self.possible_values)
    }

    pub fn allows(&self, value: usize) -> bool {
        if value == 0 || value > 9 {
            panic!("Value out of range");
        }
        (self.possible_values >> value) & 1 == 1
    }

    pub fn restrict_to(&mut self, possible_values: u64) {
        self.possible_values &= possible_values;
    }

    pub fn remove(&mut self, removed_values: u64) {
        self.possible_values &= !removed_values;
    }

    pub fn fold_possible_sums(&self, sums: u64) -> u64 {
        let mut output = 0;
        let mut possible_values = self.possible_values;
        for i in 1..=9 {
            possible_values >>= 1;
            if possible_values & 1 == 1 {
                output |= sums << i;
            }
        }
        output
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            possible_values: (1 << 10) - 2,
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            (1..=9)
                .filter(|index| self.allows(*index))
                .collect::<Vec<usize>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let mut c = Cell::default();
        assert_eq!(c.get_solution(), None);
        assert!(c.allows(1));
        assert!(c.allows(3));
        c.restrict_to(1 << 3);
        assert_eq!(c.get_solution(), Some(3));
        assert!(!c.allows(1));
        assert!(c.allows(3));
    }

    #[test]
    fn test_cell_fold_sums() {
        let c1 = Cell {
            possible_values: 0x01e, // 1..4
        };
        let c2 = Cell {
            possible_values: 0x1e0, // 5..8
        };
        let sum = c2.fold_possible_sums(c1.fold_possible_sums(1)); // 6..12
        assert_eq!(sum, 0x1fc0);

        let it = vec![c1, c2].iter();
    }
}
