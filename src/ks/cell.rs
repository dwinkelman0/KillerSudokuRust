// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::util::{onehot, popcnt64};
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq)]
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

    pub fn num_possible_solutions(&self) -> usize {
        popcnt64(self.possible_values)
    }

    pub fn restrict_to(&self, possible_values: u64) -> Self {
        Cell {
            possible_values: self.possible_values & possible_values,
        }
    }

    pub fn pairwise_restriction(&self, other: Cell, possible_sums: u64) -> Cell {
        self.possible_values().fold(*self, |output, cell_value| {
            let sums = possible_sums >> cell_value;
            let restricted = other.restrict_to(!(1 << cell_value));
            if (restricted.possible_values & sums) == 0 {
                output.restrict_to(!(1 << cell_value))
            } else {
                output
            }
        })
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

    pub fn possible_values(&self) -> PossibleValues {
        PossibleValues::new(self.possible_values)
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
        write!(f, "{:?}", self.possible_values().collect::<Vec<usize>>())
    }
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
    use super::*;

    #[test]
    fn test_cell() {
        let mut c = Cell::default();
        assert_eq!(c.get_solution(), None);
        assert!(c.allows(1));
        assert!(c.allows(3));
        c = c.restrict_to(1 << 3);
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
    }

    #[test]
    fn test_possible_values_iterator() {
        let possible_values = PossibleValues::new(0x108f);
        assert_eq!(
            possible_values.collect::<Vec<usize>>(),
            vec![0, 1, 2, 3, 7, 12]
        );
    }
}
