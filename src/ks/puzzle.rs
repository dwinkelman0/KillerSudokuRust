// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::{
    cage::Cage, cell::Cell, io::SerializablePuzzle, util::get_population_distribution,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    thread,
};

#[derive(Clone)]
pub struct Puzzle {
    pub board: [Cell; 81],
    cages: BTreeSet<Cage>,
}

impl Puzzle {
    pub fn new() -> Self {
        let mut output = Self {
            board: [Cell::default(); 81],
            cages: BTreeSet::new(),
        };
        for i in 0..9 {
            output
                .cages
                .insert(Cage::new(((i * 9)..((i + 1) * 9)).collect(), 45, true));
        }
        for i in 0..9 {
            output
                .cages
                .insert(Cage::new((0..9).map(|j| j * 9 + i).collect(), 45, true));
        }
        for i in 0..3 {
            for j in 0..3 {
                output.cages.insert(Cage::new(
                    (0..3)
                        .map(|ii| (0..3).map(move |jj| (i * 3 + ii) * 9 + (j * 3 + jj)))
                        .flatten()
                        .collect(),
                    45,
                    true,
                ));
            }
        }
        output
    }

    pub fn from_serializable(s: SerializablePuzzle) -> Self {
        let mut output = Puzzle::new();
        output.init_cages(
            s.cages
                .into_iter()
                .map(|cage| (cage.sum, cage.cell_indices))
                .collect(),
            true,
        );
        output
    }

    fn derive_cages(&mut self) {
        const MAX_DERIVED_CAGE_SIZE: usize = 8;
        /* For each top-level cage, see which other cages are completely contained or overlap */
        let derive_cages = |parent_cage: &Cage| -> Vec<Cage> {
            let mut parent_cage = parent_cage.clone();
            let mut excess_cage = Cage::empty();
            for child_cage in self.cages.iter().filter(|cage| cage.cells.len() < 9) {
                let (intersection, parent_difference, child_difference) =
                    parent_cage.get_intersection_and_difference(child_cage);
                if child_difference.len() == 0 {
                    /* The child is contained within the parent */
                    parent_cage =
                        Cage::new(parent_difference, parent_cage.sum - child_cage.sum, true);
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
                    false,
                ));
            }
            if 0 < parent_cage.cells.len() && parent_cage.cells.len() <= MAX_DERIVED_CAGE_SIZE {
                /* There are some cells leftover from the parent cage */
                output.push(parent_cage);
            }
            output
        };

        let mut new_cages = self
            .cages
            .iter()
            .filter(|cage| cage.cells.len() == 9)
            .map(|cage| derive_cages(cage))
            .flatten()
            .collect::<BTreeSet<Cage>>();
        let mut cage_len_count = BTreeMap::new();
        for cage in new_cages.iter() {
            *cage_len_count.entry(cage.cells.len()).or_insert(0) += 1;
        }
        // println!(
        //     "Derived {} new cages ({})",
        //     new_cages.len(),
        //     cage_len_count
        //         .iter()
        //         .map(|(len, count)| { format!("{count} cages with {len} cells") })
        //         .collect::<Vec<String>>()
        //         .join(", "),
        // );
        self.cages.append(&mut new_cages);
    }

    pub fn init_cages(&mut self, cages: Vec<(usize, Vec<usize>)>, perform_checks: bool) {
        for (sum, cells) in cages {
            self.cages.insert(Cage::new(cells, sum, true));
        }
        if perform_checks {
            let check = self.check_cages(4);
            if check.len() > 0 {
                panic!("Cells in cages are not balanced: {:?}", check);
            }
            self.derive_cages();
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

    /// Get the distribution of cages by size
    #[allow(unused)]
    pub fn get_cage_size_distribution(&self) -> BTreeMap<usize, usize> {
        get_population_distribution(&mut self.cages.iter(), |cage: &Cage| cage.cells.len())
    }

    /// For each cell, take the size of the smallest cage of which it is a member, and aggregate
    #[allow(unused)]
    pub fn get_cell_solvability_distribution(&self) -> BTreeMap<usize, usize> {
        let mut minimal_cage_size = vec![9; 81];
        for cage in self.cages.iter() {
            for cell in cage.cells.iter() {
                minimal_cage_size[*cell] = minimal_cage_size[*cell].min(cage.cells.len());
            }
        }
        get_population_distribution(&mut minimal_cage_size.iter(), |x| *x)
    }

    fn reduce_by_combination(&mut self) -> Result<bool, ()> {
        self.cages.iter().try_fold(false, |progress, cage| {
            Ok(cage.restrict_by_combination(&mut self.board)? | progress)
        })
    }

    fn reduce_by_partition(&mut self) -> Result<bool, ()> {
        let mut progress = false;
        loop {
            let substitutions = self.cages.iter().try_fold(vec![], |mut accum, cage| {
                if let Some(res) = cage.check_for_partitions(&mut self.board)? {
                    accum.push((cage.clone(), res))
                }
                Ok(accum)
            })?;
            if substitutions.len() > 0 {
                progress = true;
                substitutions.into_iter().for_each(
                    |(original_cage, (new_cage, remaining_cage))| {
                        self.cages.remove(&original_cage);
                        self.cages.insert(new_cage);
                        self.cages.insert(remaining_cage);
                    },
                );
            } else {
                break;
            }
        }
        Ok(progress)
    }

    fn solve_until_stuck(&mut self) -> Result<bool, ()> {
        self.cages.iter().try_for_each(|cage| {
            cage.restrict_by_uniform_combination(&mut self.board)?;
            Ok(())
        })?;
        self.reduce_by_partition()?;
        while self.reduce_by_combination()? {
            self.reduce_by_partition()?;
        }

        /* Final return value */
        Ok(self.board.iter().all(|cell| cell.get_solution().is_some()))
    }

    fn solve_until_stuck_then_guess_and_fork(&mut self, depth: usize) -> Result<Vec<Puzzle>, ()> {
        const RECURSION_LIMIT: usize = 10;
        if self.solve_until_stuck()? {
            Ok(vec![self.clone()])
        } else if depth > RECURSION_LIMIT {
            Err(())
        } else {
            /* Figure out how many cells each cell influences */
            let mut cage_count = vec![BTreeSet::new(); 81];
            for cage in self.cages.iter() {
                if cage.uniqueness {
                    for cell_index_a in cage.cells.iter() {
                        for cell_index_b in cage.cells.iter() {
                            cage_count[*cell_index_a].insert(*cell_index_b);
                        }
                    }
                }
            }
            let guess_index = cage_count
                .into_iter()
                .enumerate()
                .filter(|(i, _)| self.board[*i].get_solution().is_none())
                .max_by_key(|(i, cells)| cells.len() / self.board[*i].num_possible_solutions())
                .map(|(i, _)| i)
                .unwrap();
            let res = self.board[guess_index]
                .possible_values()
                .map(|guess_value| {
                    let guess_index = guess_index;
                    let mut puzzle_copy = self.clone();
                    thread::spawn(move || {
                        puzzle_copy.board[guess_index]
                            .restrict_to(1 << guess_value)
                            .unwrap();
                        puzzle_copy.solve_until_stuck_then_guess_and_fork(depth + 1)
                    })
                })
                .filter_map(|handle| handle.join().unwrap().ok())
                .flatten()
                .collect();
            Ok(res)
        }
    }

    pub fn solve(&self) -> Result<Vec<Puzzle>, ()> {
        let mut working_copy = self.clone();
        working_copy.solve_until_stuck_then_guess_and_fork(0)
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
