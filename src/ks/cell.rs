// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::combinations::PossibleValues;
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

    pub fn get_bits(&self) -> u64 {
        self.possible_values
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

    pub fn restrict_to(&mut self, possible_values: u64) -> Result<(), ()> {
        self.possible_values &= possible_values;
        if self.possible_values == 0 {
            Err(())
        } else {
            Ok(())
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let mut c = Cell::default();
        assert_eq!(c.get_solution(), None);
        assert!(c.allows(1));
        assert!(c.allows(3));
        c.restrict_to(1 << 3).unwrap();
        assert_eq!(c.get_solution(), Some(3));
        assert!(!c.allows(1));
        assert!(c.allows(3));
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
