// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::{cage::Cage, cell::Cell};
use std::fmt::Display;

pub struct Puzzle {
    pub board: [Cell; 81],
    cages: Vec<Cage>,
}

impl Puzzle {
    pub fn new() -> Self {
        let mut output = Self {
            board: [Cell::default(); 81],
            cages: vec![],
        };
        for i in 0..9 {
            output.cages.push(Cage {
                cells: ((i * 9)..((i + 1) * 9)).collect(),
                sum: 45,
            });
        }
        for i in 0..9 {
            output.cages.push(Cage {
                cells: (0..9).map(|j| j * 9 + i).collect(),
                sum: 45,
            });
        }
        for i in 0..3 {
            for j in 0..3 {
                output.cages.push(Cage {
                    cells: (0..3)
                        .map(|ii| (0..3).map(move |jj| (i * 3 + ii) * 9 + (j * 3 + jj)))
                        .flatten()
                        .collect(),
                    sum: 45,
                })
            }
        }
        output
    }

    pub fn init_cages(&mut self, cages: Vec<(usize, Vec<usize>)>) {
        for (sum, cells) in cages {
            self.cages.push(Cage { cells, sum });
        }
        let check = self.check_cages(4);
        if check.len() > 0 {
            panic!("Cells in cages are not balanced: {:?}", check);
        }
    }

    fn check_cages(&self, expected: usize) -> Vec<(usize, usize)> {
        let mut sums: [usize; 81] = [0; 81];
        for cage in self.cages.iter() {
            for cell in cage.cells.iter() {
                sums[*cell] += 1;
            }
        }
        sums.iter()
            .enumerate()
            .filter_map(|(index, sum)| (*sum != expected).then_some((index, *sum)))
            .collect()
    }

    pub fn solve(&mut self) -> bool {
        loop {
            // TODO: turn off short-circuit evalution
            let any_progress = self.cages.iter_mut().fold(false, |any_progress, cage| {
                any_progress || cage.restrict(&mut self.board)
            });
            match any_progress {
                true => println!("Made progress"),
                false => break,
            }
        }

        /* Final return value */
        self.board.iter().all(|cell| cell.get_solution().is_some())
    }
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---- Cells ----")?;
        for (i, cell) in self.board.iter().enumerate() {
            writeln!(f, "{:>2}: {}", i, cell)?;
        }
        writeln!(f, "---- Cages ----")?;
        for cage in self.cages.iter() {
            writeln!(f, "{cage}")?;
        }
        Ok(())
    }
}
