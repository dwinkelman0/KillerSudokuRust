// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::{cage::Cage, cell::Cell};
use std::{collections::BTreeMap, fmt::Display};

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
            output
                .cages
                .push(Cage::new(((i * 9)..((i + 1) * 9)).collect(), 45));
        }
        for i in 0..9 {
            output
                .cages
                .push(Cage::new((0..9).map(|j| j * 9 + i).collect(), 45));
        }
        for i in 0..3 {
            for j in 0..3 {
                output.cages.push(Cage::new(
                    (0..3)
                        .map(|ii| (0..3).map(move |jj| (i * 3 + ii) * 9 + (j * 3 + jj)))
                        .flatten()
                        .collect(),
                    45,
                ))
            }
        }
        output
    }

    fn derive_cages(&mut self) {
        /* For each top-level cage, see which other cages are completely contained or overlap */
        const TOP_LEVEL_CAGES: std::ops::Range<usize> = 0..27;
        let derive_cages = |parent_cage: &Cage| -> Vec<Cage> {
            let mut parent_cage = parent_cage.clone();
            let mut excess_cage = Cage::empty();
            for child_cage_index in 27..self.cages.len() {
                let child_cage = &self.cages[child_cage_index];
                let (intersection, parent_difference, child_difference) =
                    parent_cage.get_intersection_and_difference(child_cage);
                if child_difference.len() == 0 {
                    /* The child is contained within the parent */
                    parent_cage = Cage::new(parent_difference, parent_cage.sum - child_cage.sum);
                } else if intersection.len() > 0 {
                    /* The child at least partially overlaps the parent */
                    excess_cage = excess_cage.merge(child_cage);
                }
            }
            let (_, excess_difference, parent_difference) =
                excess_cage.get_intersection_and_difference(&parent_cage);
            /* There should be no parent cells remaining not included in the excess cage */
            assert_eq!(parent_difference.len(), 0);
            let mut output = vec![];
            if 0 < excess_difference.len() && excess_difference.len() <= 4 {
                /* There are cells that extend beyond the parent cage */
                output.push(Cage::new(
                    excess_difference,
                    excess_cage.sum - parent_cage.sum,
                ));
            }
            if 0 < parent_cage.cells.len() && parent_cage.cells.len() <= 4 {
                /* There are some cells leftover from the parent cage */
                output.push(parent_cage);
            }
            output
        };

        let mut new_cages = TOP_LEVEL_CAGES
            .map(|top_level_cage_index| derive_cages(&self.cages[top_level_cage_index]))
            .flatten()
            .collect::<Vec<Cage>>();
        let mut cage_len_count = BTreeMap::new();
        for cage in new_cages.iter() {
            *cage_len_count.entry(cage.cells.len()).or_insert(0) += 1;
        }
        println!(
            "Derived {} new cages ({})",
            new_cages.len(),
            cage_len_count
                .iter()
                .map(|(len, count)| { format!("{count} cages with {len} cells") })
                .collect::<Vec<String>>()
                .join(", "),
        );
        let initial_complexity = self.get_cell_solvability_distribution();
        self.cages.append(&mut new_cages);
        let final_complexity = self.get_cell_solvability_distribution();
        println!("Cell solvability distribution changed from {initial_complexity:?} to {final_complexity:?}");
    }

    pub fn init_cages(&mut self, cages: Vec<(usize, Vec<usize>)>) {
        for (sum, cells) in cages {
            self.cages.push(Cage::new(cells, sum));
        }
        let check = self.check_cages(4);
        if check.len() > 0 {
            panic!("Cells in cages are not balanced: {:?}", check);
        }
        self.derive_cages();
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

    /// For each cell, take the size of the smallest cage of which it is a member, and aggregate
    pub fn get_cell_solvability_distribution(&self) -> BTreeMap<usize, usize> {
        let mut minimal_cage_size = vec![9; 81];
        for cage in self.cages.iter() {
            for cell in cage.cells.iter() {
                minimal_cage_size[*cell] = minimal_cage_size[*cell].min(cage.cells.len());
            }
        }
        let mut cage_size_count = BTreeMap::new();
        for size in minimal_cage_size {
            *cage_size_count.entry(size).or_insert(0) += 1;
        }
        cage_size_count
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
