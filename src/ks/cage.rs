// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::fmt::Display;

use crate::ks::cell::Cell;

pub struct Cage {
    pub cells: Vec<usize>,
    pub sum: usize,
}

impl Cage {
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
                        board[cell_index].restrict_to(remaining_numbers);
                        board[cell_index]
                            .get_solution()
                            .is_none()
                            .then(|| (cell_index, board[cell_index]))
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
        write!(f, "{:?}", self.cells)
    }
}
