// Copyright 2022 by Daniel Winkelman. All rights reserved.

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
        println!(
            "Restrict to {:?}",
            PossibleValues::new(combinations_union).collect::<Vec<usize>>()
        );
        self.cells.iter().for_each(|cell_index| {
            board[*cell_index] = board[*cell_index].restrict_to(combinations_union);
        });
        self.get_degrees_of_freedom(board) < init_degrees_of_freedom
    }

    /// Returns new partitions if this cage can be split into smaller cages
    pub fn check_for_partition(&self, board: &mut [Cell; 81]) -> Option<Vec<Cage>> {
        let mut output = vec![];
        let mut candidates: Vec<(u64, BTreeSet<usize>)> = vec![];
        let mut sorted_cells = self
            .cells
            .iter()
            .map(|cell_index| (*cell_index, board[*cell_index].get_bits()))
            .collect::<Vec<(usize, u64)>>();
        sorted_cells.sort_by_key(|(_, bits)| popcnt64(*bits));
        for (cell_index, possible_values) in sorted_cells {
            /* Update candidates list */
            let mut new_candidate = (possible_values, BTreeSet::from([cell_index]));
            let matching_candidates = candidates
                .iter()
                .enumerate()
                .filter_map(|(candidate_index, (candidate_values, _))| {
                    ((possible_values & candidate_values) != 0).then_some(candidate_index)
                })
                .collect::<Vec<usize>>();
            matching_candidates.iter().for_each(|i| {
                new_candidate.0 |= candidates[*i].0;
                new_candidate.1 = new_candidate.1.union(&candidates[*i].1).cloned().collect();
            });
            matching_candidates.iter().rev().for_each(|i| {
                candidates.remove(*i);
            });
            candidates.push(new_candidate);

            /* Filter out candidates that meet criteria */
            loop {
                let (partition, updated_candidates) =
                    candidates
                        .into_iter()
                        .partition(|(possible_values, cell_indices)| {
                            popcnt64(*possible_values) == cell_indices.len()
                        });
                candidates = updated_candidates;
                match partition.len() {
                    0 => break,
                    1 => {
                        let new_partition = partition.into_iter().nth(0).unwrap();
                        for candidate in candidates.iter_mut() {
                            candidate.0 &= !new_partition.0;
                        }
                        output.push(new_partition);
                    }
                    _ => panic!("Invalid condition"),
                }
            }
        }
        match (output.len(), candidates.len()) {
            (0, _) => None,
            (1, 0) => None,
            (_, 0..=1) => {
                /* Create new cages from known partitions */
                let mut running_total = 0;
                let mut disallowed_values = 0;
                let mut new_cages = output
                    .into_iter()
                    .map(|(possible_values, cell_indices)| {
                        disallowed_values |= possible_values;
                        let sum = PossibleValues::new(possible_values).sum::<usize>();
                        running_total += sum;
                        Cage::new(cell_indices.into_iter().collect(), sum)
                    })
                    .collect::<Vec<Cage>>();
                if let Some((_, leftover_cell_indices)) = candidates.into_iter().nth(0) {
                    for cell_index in leftover_cell_indices.iter() {
                        board[*cell_index] = board[*cell_index].restrict_to(!disallowed_values);
                    }
                    new_cages.push(Cage::new(
                        leftover_cell_indices.into_iter().collect(),
                        self.sum - running_total,
                    ));
                }
                Some(new_cages)
            }
            (_, _) => panic!("Invalid condition"),
        }
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
