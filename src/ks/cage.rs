// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::fmt::Display;

use crate::ks::cell::Cell;
use crate::ks::combinations::{cage_can_have_uniqueness, get_combinations_union, PossibleValues};
use crate::ks::util::popcnt64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cage {
    pub cells: Vec<usize>,
    pub sum: usize,
    pub uniqueness: bool,
}

impl Cage {
    pub fn new(cells: Vec<usize>, sum: usize, uniqueness: bool) -> Self {
        let uniqueness = uniqueness || cage_can_have_uniqueness(&cells);
        let mut output = Self {
            cells,
            sum,
            uniqueness,
        };
        output.cells.sort();
        output
    }

    pub fn empty() -> Self {
        Cage {
            cells: vec![],
            sum: 0,
            uniqueness: false,
        }
    }

    fn get_degrees_of_freedom(&self, board: &[Cell; 81]) -> usize {
        self.cells
            .iter()
            .map(|cell_index| board[*cell_index].num_possible_solutions())
            .sum()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut output = self.clone();
        other
            .cells
            .iter()
            .for_each(|cell_index| output.cells.push(*cell_index));
        output.cells.sort();
        output.sum += other.sum;
        output
    }

    /// let A = self, B = other; returns (A intersect B, A - B, B - A)
    pub fn get_intersection_and_difference(
        &self,
        other: &Self,
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let mut intersection = vec![];
        let mut difference_a = vec![];
        let mut difference_b = vec![];
        let mut a_it = self.cells.iter();
        let mut b_it = other.cells.iter();
        let mut a = a_it.next();
        let mut b = b_it.next();
        while let (Some(a_value), Some(b_value)) = (a, b) {
            match a_value.cmp(b_value) {
                std::cmp::Ordering::Equal => {
                    intersection.push(*a_value);
                    a = a_it.next();
                    b = b_it.next();
                }
                std::cmp::Ordering::Less => {
                    difference_a.push(*a_value);
                    a = a_it.next();
                }
                std::cmp::Ordering::Greater => {
                    difference_b.push(*b_value);
                    b = b_it.next();
                }
            }
        }
        while let Some(a_value) = a {
            difference_a.push(*a_value);
            a = a_it.next();
        }
        while let Some(b_value) = b {
            difference_b.push(*b_value);
            b = b_it.next();
        }
        (intersection, difference_a, difference_b)
    }

    /// Returns true if progress was made
    pub fn restrict_by_uniform_combination(&self, board: &mut [Cell; 81]) -> Result<bool, ()> {
        let init_degrees_of_freedom = self.get_degrees_of_freedom(board);
        let combinations_union = get_combinations_union(self.cells.len(), self.sum);
        self.cells
            .iter()
            .try_for_each(|cell_index| board[*cell_index].restrict_to(combinations_union))?;
        Ok(self.get_degrees_of_freedom(board) < init_degrees_of_freedom)
    }

    pub fn check_for_partitions(&self, board: &mut [Cell; 81]) -> Result<Option<(Cage, Cage)>, ()> {
        if !self.uniqueness {
            return Ok(None);
        }

        fn fold_combinations(
            index: usize,
            choose: usize,
            max_value_len: usize,
            data: &[(usize, u64)],
        ) -> Vec<(u64, u64)> {
            match choose {
                0 => panic!("Invalid condition"),
                1 => (index..data.len())
                    .filter_map(|i| {
                        let (key, value) = data[i];
                        (popcnt64(value) <= max_value_len).then_some((1 << key, value))
                    })
                    .collect(),
                choose => (index..(data.len() - choose - 1))
                    .map(|i| {
                        fold_combinations(i + 1, choose - 1, max_value_len, data)
                            .iter()
                            .filter_map(|(folded_key, folded_value)| {
                                let (key, value) = data[i];
                                let updated_value = folded_value | value;
                                (popcnt64(updated_value) <= max_value_len)
                                    .then_some((folded_key | (1 << key), updated_value))
                            })
                            .collect::<Vec<(u64, u64)>>()
                    })
                    .flatten()
                    .collect(),
            }
        }

        let gather_cell_indices = |cells: u64, is_positive: bool| {
            self.cells
                .iter()
                .enumerate()
                .filter_map(|(i, cell_index)| {
                    ((((cells >> i) & 1) == 1) ^ !is_positive).then_some(*cell_index)
                })
                .collect::<Vec<usize>>()
        };

        let possible_values_by_cell = {
            let mut v = self
                .cells
                .iter()
                .map(|cell_index| board[*cell_index].get_bits())
                .enumerate()
                .collect::<Vec<(usize, u64)>>();
            v.sort_by_key(|(_, possible_values)| popcnt64(*possible_values));
            v
        };
        for i in 1..=(possible_values_by_cell.len() - 1) {
            if let Some((cells, values)) = fold_combinations(0, i, i, &possible_values_by_cell)
                .first()
                .cloned()
            {
                assert_eq!(popcnt64(cells), popcnt64(values));
                let new_cage_cells = gather_cell_indices(cells, true);
                for cell in new_cage_cells.iter() {
                    board[*cell].restrict_to(values)?;
                }
                let new_cage_sum = PossibleValues::new(values).sum::<usize>();
                let new_cage = Cage::new(new_cage_cells, new_cage_sum, self.uniqueness);
                let remaining_cage_cells = gather_cell_indices(cells, false);
                for cell in remaining_cage_cells.iter() {
                    board[*cell].restrict_to(!values)?;
                }
                let remaining_cage_sum = self.sum - new_cage_sum;
                let remaining_cage =
                    Cage::new(remaining_cage_cells, remaining_cage_sum, self.uniqueness);
                remaining_cage.restrict_by_uniform_combination(board)?;
                return Ok(Some((new_cage, remaining_cage)));
            }
        }

        let possible_cells_by_value = {
            let mut v = (1..=9)
                .map(|value| {
                    (
                        value,
                        self.cells
                            .iter()
                            .enumerate()
                            .fold(0, |accum, (i, cell_index)| {
                                if board[*cell_index].allows(value) {
                                    accum | (1 << i)
                                } else {
                                    accum
                                }
                            }),
                    )
                })
                .filter(|(_, possible_cells)| popcnt64(*possible_cells) > 0)
                .collect::<Vec<(usize, u64)>>();
            v.sort_by_key(|(_, possible_cells)| popcnt64(*possible_cells));
            v
        };
        if possible_cells_by_value.len() > self.cells.len() {
            return Ok(None);
        }
        for i in 1..=(possible_cells_by_value.len() - 1) {
            if let Some((values, cells)) = fold_combinations(0, i, i, &possible_cells_by_value)
                .first()
                .cloned()
            {
                assert_eq!(popcnt64(cells), popcnt64(values));
                let new_cage_cells = gather_cell_indices(cells, true);
                for cell in new_cage_cells.iter() {
                    board[*cell].restrict_to(values)?;
                }
                let new_cage_sum = PossibleValues::new(values).sum::<usize>();
                let new_cage = Cage::new(new_cage_cells, new_cage_sum, self.uniqueness);
                let remaining_cage_cells = gather_cell_indices(cells, false);
                for cell in remaining_cage_cells.iter() {
                    board[*cell].restrict_to(!values)?;
                }
                let remaining_cage_sum = self.sum - new_cage_sum;
                let remaining_cage =
                    Cage::new(remaining_cage_cells, remaining_cage_sum, self.uniqueness);
                remaining_cage.restrict_by_uniform_combination(board)?;
                return Ok(Some((new_cage, remaining_cage)));
            }
        }

        Ok(None)
    }

    /// Returns true if progress was made
    pub fn restrict_by_combination(&self, board: &mut [Cell; 81]) -> Result<bool, ()> {
        match self.cells.len() {
            0 => panic!("Invalid condition"),
            1 => Ok(false),
            2 => {
                let get_complement_bits = |mask: u64| {
                    (mask.reverse_bits() >> (64 - self.sum - 1)) & ((1 << self.sum) - 2)
                };
                let a_mask = board[self.cells[0]].get_bits();
                let b_mask = board[self.cells[1]].get_bits();
                board[self.cells[1]].restrict_to(get_complement_bits(a_mask))?;
                board[self.cells[0]].restrict_to(get_complement_bits(b_mask))?;
                Ok((a_mask != board[self.cells[0]].get_bits())
                    || (b_mask != board[self.cells[1]].get_bits()))
            }
            _ => Ok(false),
        }
    }
}

impl Display for Cage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {}", self.cells, self.sum)
    }
}
