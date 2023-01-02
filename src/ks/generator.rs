// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::{
    io::{SerializableCage, SerializablePuzzle},
    puzzle::Puzzle,
};

use std::collections::{BTreeMap, BTreeSet};

use rand::{
    seq::{self, SliceRandom},
    thread_rng,
};

#[derive(Debug)]
pub struct Generator {
    numbers: [usize; 81],
    cages: [usize; 81],
    remaining_merges: Vec<(usize, usize)>,
}

impl Generator {
    /// Initialize the generator with the canonical solution
    pub fn new() -> Self {
        let mut rng = thread_rng();
        // macro_rules! vec_shuffled {
        //     [$($x:expr,)*] => {
        //         {
        //             let mut v = vec![$($x,)*];
        //             v.shuffle(&mut rng);
        //             v
        //         }
        //     };
        // }
        // let solution = [0; 81];
        // {
        //     let mut puzzle = Puzzle::new();
        //     let mut top_row = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        //     let mut middle_left = vec![4, 6, 8];
        //     let mut bottom_left = vec![5, 7, 9];
        //     let mut combo = vec_shuffled![1, 2, 3,];
        //     let mut middle_center = vec_shuffled![5, 7, combo[0],];
        //     let mut middle_right = vec_shuffled![9, combo[1], combo[2],];
        //     let mut bottom_center = vec_shuffled![4, combo[1], combo[2],];
        //     let mut bottom_right = vec_shuffled![6, 8, combo[0],];
        //     let mut output = top_row;
        //     output.append(&mut middle_left);
        //     output.append(&mut middle_center);
        //     output.append(&mut middle_right);
        //     output.append(&mut bottom_left);
        //     output.append(&mut bottom_center);
        //     output.append(&mut bottom_right);
        //     let cages = output
        //         .into_iter()
        //         .enumerate()
        //         .map(|(i, value)| (value, vec![i]))
        //         .collect::<Vec<(usize, Vec<usize>)>>();
        //     puzzle.init_cages(cages, false);
        //     println!("{:?}", puzzle.solve().map(|solutions| solutions.len()));
        // }
        let mut canonical_solution = [0; 81];
        for row in 0..9 {
            let offset = row * 3 + row / 3;
            for col in 0..9 {
                canonical_solution[row * 9 + col] = (col + offset) % 9 + 1;
            }
        }
        let mut initial_cages = [0; 81];
        for i in 0..81 {
            initial_cages[i] = i;
        }
        let mut merges = vec![];
        for row in 0..9 {
            for col in 0..8 {
                merges.push((row * 9 + col, row * 9 + col + 1));
                merges.push((col * 9 + row, (col + 1) * 9 + row));
            }
        }
        merges.shuffle(&mut rng);
        let mut output = Self {
            numbers: canonical_solution,
            cages: initial_cages,
            remaining_merges: merges,
        };
        for _ in 0..5 {
            output.renumber();
            output.shuffle_rows();
            output.partial_resolution();
        }
        output
    }

    fn renumber(&mut self) {
        let sequence = {
            let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
            v.shuffle(&mut thread_rng());
            v
        };
        self.numbers
            .iter_mut()
            .for_each(|value| *value = sequence[*value - 1]);
    }

    fn shuffle_rows(&mut self) {
        let sequence = vec![3, 5, 4, 6, 7, 8, 0, 2, 1];
        let mut new_numbers = [0; 81];
        sequence.into_iter().enumerate().for_each(|(row, seq)| {
            for i in 0..9 {
                new_numbers[row * 9 + i] = self.numbers[seq * 9 + i];
            }
        });
        self.numbers = new_numbers;
    }

    fn partial_resolution(&mut self) {
        /* Choose three random numbers */
        let mut rng = thread_rng();
        let numbers = {
            let mut first = vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
                .choose_multiple(&mut rng, 4)
                .cloned()
                .collect::<Vec<usize>>();
            first.sort();
            first
        };

        /* Eliminate chosen numbers from part of the puzzle */
        let cages = self
            .numbers
            .iter()
            .enumerate()
            .filter_map(|(i, value)| {
                if i < 27 {
                    Some((*value, vec![i]))
                } else {
                    match numbers.iter().find(|n| *n == value) {
                        Some(_) => None,
                        None => Some((*value, vec![i])),
                    }
                }
            })
            .collect();

        /* Find all solutions (there should be several) and chose a random one */
        let mut puzzle = Puzzle::new();
        puzzle.init_cages(cages, false);
        let p = puzzle.solve().unwrap().choose(&mut rng).unwrap().clone();
        self.numbers.iter_mut().enumerate().for_each(|(i, value)| {
            *value = p.board[i].get_solution().unwrap();
        });
    }

    pub fn serialize(&self) -> SerializablePuzzle {
        SerializablePuzzle {
            cell_values: self.numbers.iter().cloned().collect(),
            cages: self
                .cages
                .iter()
                .enumerate()
                .fold(BTreeMap::new(), |mut accum, (cell_index, cage_index)| {
                    let e = accum.entry(*cage_index).or_insert((0, vec![]));
                    e.0 += self.numbers[cell_index];
                    e.1.push(cell_index);
                    accum
                })
                .into_iter()
                .map(|(k, (sum, cell_indices))| SerializableCage { sum, cell_indices })
                .collect(),
        }
    }

    fn attempt_merge(&mut self) -> bool {
        true
    }
}
