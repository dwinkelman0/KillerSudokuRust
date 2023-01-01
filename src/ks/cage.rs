// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::collections::btree_map::Values;
use std::collections::BTreeSet;
use std::fmt::Display;

use crate::ks::cell::Cell;
use crate::ks::combinations::{get_combinations_union, PossibleValues};
use crate::ks::util::popcnt64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cage {
    pub cells: Vec<usize>,
    pub sum: usize,
}

impl Cage {
    pub fn new(cells: Vec<usize>, sum: usize) -> Self {
        let mut output = Self { cells, sum };
        output.cells.sort();
        output
    }

    pub fn empty() -> Self {
        Cage {
            cells: vec![],
            sum: 0,
        }
    }

    pub fn get_possible_sums(&self, board: &[Cell; 81]) -> u64 {
        self.cells
            .iter()
            .fold(1, |accum, cell| board[*cell].fold_possible_sums(accum))
    }

    fn get_remaining_sum(&self, board: &[Cell; 81]) -> usize {
        self.cells.iter().fold(self.sum, |accum, cell_index| {
            accum - board[*cell_index].get_solution().unwrap_or(0)
        })
    }

    fn get_remaining_numbers(&self, board: &[Cell; 81]) -> u64 {
        self.cells.iter().fold((1 << 10) - 1, |accum, cell_index| {
            match board[*cell_index].get_solution() {
                Some(solution) => accum & !(1 << solution),
                None => accum,
            }
        })
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
    pub fn restrict_by_uniform_combination(&self, board: &mut [Cell; 81]) -> bool {
        let init_degrees_of_freedom = self.get_degrees_of_freedom(board);
        let combinations_union = get_combinations_union(self.cells.len(), self.sum);
        self.cells.iter().for_each(|cell_index| {
            board[*cell_index] = board[*cell_index].restrict_to(combinations_union);
        });
        self.get_degrees_of_freedom(board) < init_degrees_of_freedom
    }

    pub fn check_for_partitions(&self, board: &mut [Cell; 81]) -> Option<(Cage, Cage)> {
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
                let new_cage_cells = self
                    .cells
                    .iter()
                    .enumerate()
                    .filter_map(|(i, cell_index)| (((cells >> i) & 1) == 1).then_some(*cell_index))
                    .collect::<Vec<usize>>();
                for cell in new_cage_cells.iter() {
                    board[*cell] = board[*cell].restrict_to(values);
                }
                let new_cage_sum = PossibleValues::new(values).sum::<usize>();
                let new_cage = Cage::new(new_cage_cells, new_cage_sum);
                let remaining_cage_cells = self
                    .cells
                    .iter()
                    .enumerate()
                    .filter_map(|(i, cell_index)| (((cells >> i) & 1) == 0).then_some(*cell_index))
                    .collect::<Vec<usize>>();
                for cell in remaining_cage_cells.iter() {
                    board[*cell] = board[*cell].restrict_to(!values);
                }
                let remaining_cage_sum = self.sum - new_cage_sum;
                let remaining_cage = Cage::new(remaining_cage_cells, remaining_cage_sum);
                remaining_cage.restrict_by_uniform_combination(board);
                return Some((new_cage, remaining_cage));
            }
        }

        // let possible_cells_by_value = {
        //     let mut v = (1..=9)
        //         .map(|value| {
        //             self.cells
        //                 .iter()
        //                 .enumerate()
        //                 .fold(0, |accum, (i, cell_index)| {
        //                     if board[*cell_index].allows(value) {
        //                         accum | (1 << i)
        //                     } else {
        //                         accum
        //                     }
        //                 })
        //         })
        //         .enumerate()
        //         .filter(|(_, possible_cells)| popcnt64(*possible_cells) > 0)
        //         .collect::<Vec<(usize, u64)>>();
        //     v.sort_by_key(|(_, possible_cells)| popcnt64(*possible_cells));
        //     v
        // };
        None
    }

    /// Returns true if progress was made
    fn restrict_subset(
        &self,
        current_cell: Cell,
        unsolved_cells: &[Cell],
        remaining_sum: u64,
        remaining_numbers: u64,
    ) -> Vec<Cell> {
        match unsolved_cells.len() {
            0 => {
                /* Only one cell left */
                vec![current_cell.restrict_to(remaining_sum & remaining_numbers)]
            }
            1 => {
                /* Only two cells left */
                let mut cell_a = current_cell;
                let mut cell_b = *unsolved_cells.first().unwrap();
                loop {
                    let new_cell_a = cell_a.pairwise_restriction(cell_b, remaining_sum);
                    let new_cell_b = cell_b.pairwise_restriction(cell_a, remaining_sum);
                    if new_cell_a == cell_a && new_cell_b == cell_b {
                        return vec![new_cell_a, new_cell_b];
                    } else {
                        cell_a = new_cell_a;
                        cell_b = new_cell_b;
                    }
                }
            }
            _ => {
                // Give up for now
                let mut output = vec![current_cell];
                for cell in unsolved_cells {
                    output.push(cell.clone())
                }
                output
            }
        }

        // /* Recurse */
        // match unsolved_cells.first() {
        //     Some(next_current_cell) => {
        //         let mut it = unsolved_cells.iter();
        //         it.next();
        //         self.restrict_subset(
        //             board,
        //             *next_current_cell,
        //             it.as_slice(),
        //             remaining_sum,
        //             remaining_numbers,
        //         )
        //     }
        //     None => false,
        // }
    }

    /// Returns true if progress was made
    pub fn restrict(&self, board: &mut [Cell; 81]) -> bool {
        let remaining_sum = self.get_remaining_sum(board);
        if remaining_sum == 0 {
            /* Already solved */
            false
        } else {
            /* Determine initial number of degrees of freedom */
            let initial_degrees_of_freedom = self.get_degrees_of_freedom(board);

            /* Filter cells by those that are not fully constrained */
            let remaining_numbers = self.get_remaining_numbers(board);
            let mut unsolved_cells = self
                .cells
                .clone()
                .into_iter()
                .filter_map(|cell_index| match board[cell_index].get_solution() {
                    Some(_) => None,
                    None => {
                        let restricted = board[cell_index].restrict_to(remaining_numbers);
                        restricted
                            .get_solution()
                            .is_none()
                            .then(|| (cell_index, restricted))
                    }
                })
                .collect::<Vec<(usize, Cell)>>();

            /* Recursively search solution space */
            unsolved_cells.sort_by_key(|(index, cell)| cell.num_possible_solutions());
            let cells_to_solve = unsolved_cells
                .iter()
                .map(|(_, cell)| *cell)
                .collect::<Vec<Cell>>();
            let mut it = cells_to_solve.iter();
            it.next();
            let solved_cells = self.restrict_subset(
                *cells_to_solve.first().unwrap(),
                &it.as_slice(),
                1 << remaining_sum,
                remaining_numbers,
            );

            for (i, (index, _)) in unsolved_cells.into_iter().enumerate() {
                if solved_cells[i] != board[index] {
                    println!("Updated {index} to {}", solved_cells[i]);
                }
                board[index] = solved_cells[i];
            }

            /* Check whether progress was made */
            self.get_degrees_of_freedom(board) < initial_degrees_of_freedom
        }
    }
}

impl Display for Cage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {}", self.cells, self.sum)
    }
}
