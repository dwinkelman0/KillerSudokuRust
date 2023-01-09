// Copyright 2022 by Daniel Winkelman. All rights reserved.

use crate::ks::{
    io::{SerializableCage, SerializablePuzzle},
    puzzle::Puzzle,
};

use std::collections::{BTreeMap, BTreeSet};

use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, Clone)]
struct Cage {
    cells: BTreeSet<usize>,
    adjacent_cages: BTreeSet<usize>,
}

#[derive(Debug, Clone)]
pub struct Generator {
    numbers: [usize; 81],
    cages: BTreeMap<usize, Cage>,
}

impl Generator {
    /// Initialize the generator with the canonical solution
    pub fn new_canonical() -> Self {
        let mut canonical_solution = [0; 81];
        for row in 0..9 {
            let offset = row * 3 + row / 3;
            for col in 0..9 {
                canonical_solution[row * 9 + col] = (col + offset) % 9 + 1;
            }
        }
        /* All cells initially belong to a 1-cell cage whose index matches the cell index */
        let mut cages = (0..81)
            .map(|i| {
                (
                    i,
                    Cage {
                        cells: BTreeSet::from([i]),
                        adjacent_cages: BTreeSet::new(),
                    },
                )
            })
            .collect::<BTreeMap<usize, Cage>>();
        let mut insert_merge = |a: usize, b: usize| {
            cages.get_mut(&a).unwrap().adjacent_cages.insert(b);
            cages.get_mut(&b).unwrap().adjacent_cages.insert(a);
        };
        for row in 0..9 {
            for col in 0..8 {
                insert_merge(row * 9 + col, row * 9 + col + 1);
                insert_merge(col * 9 + row, (col + 1) * 9 + row);
            }
        }
        Self {
            numbers: canonical_solution,
            cages,
        }
    }

    pub fn new_random() -> Self {
        let mut output = Self::new_canonical();
        for _ in 0..10 {
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
                .values()
                .map(|cage| SerializableCage {
                    sum: cage
                        .cells
                        .iter()
                        .map(|cell_index| self.numbers[*cell_index])
                        .sum(),
                    cell_indices: cage.cells.iter().cloned().collect(),
                })
                .collect(),
        }
    }

    fn try_merge_cages(&mut self, a: usize, b: usize) -> bool {
        /* Check that cage value uniqueness would be preserved */
        let get_cage_values = |cage_index| {
            self.cages[&cage_index]
                .cells
                .iter()
                .map(|cell_index| self.numbers[*cell_index])
                .collect::<BTreeSet<usize>>()
        };
        let a_values = get_cage_values(a);
        let b_values = get_cage_values(b);
        let ab_union_len = a_values.union(&b_values).cloned().count();
        if a_values.len() + b_values.len() == ab_union_len && ab_union_len < 8 {
            /* Insert contents of b into a */
            for adjacent_cage in self.cages[&b].adjacent_cages.clone() {
                if adjacent_cage != a {
                    self.cages
                        .get_mut(&a)
                        .unwrap()
                        .adjacent_cages
                        .insert(adjacent_cage);
                }
            }
            for cell_index in self.cages[&b].cells.clone() {
                self.cages.get_mut(&a).unwrap().cells.insert(cell_index);
            }

            /* Delete b */
            self.cages.remove(&b);

            /* Rename all references to b so that they refer to a */
            self.cages.iter_mut().for_each(|(cage_index, cage)| {
                if cage.adjacent_cages.remove(&b) {
                    if *cage_index != a {
                        cage.adjacent_cages.insert(a);
                    }
                }
            });
            true
        } else {
            /* This merge will never be valid, so scrub from adjacent cage tables */
            self.cages.get_mut(&a).unwrap().adjacent_cages.remove(&b);
            self.cages.get_mut(&b).unwrap().adjacent_cages.remove(&a);
            false
        }
    }

    fn try_merge_random_cages(&mut self) -> Result<bool, ()> {
        /* Calculate weightings for all possible merges */
        let mut possible_merges = self
            .cages
            .iter()
            .fold(BTreeMap::new(), |mut accum, (a, cage)| {
                for b in &cage.adjacent_cages {
                    let key = (a.min(b), a.max(b));
                    let num_cells = cage.cells.len() + self.cages[b].cells.len();
                    assert!(num_cells > 1);
                    let weight = 1024 / (num_cells - 1);
                    *accum.entry(key).or_insert(0) += weight;
                }
                accum
            })
            .into_iter()
            .collect::<Vec<_>>();

        if possible_merges.is_empty() {
            Err(())
        } else {
            /* Change the weight metric to the cumulative weight */
            let total_weight = possible_merges.iter_mut().fold(0, |accum, (_, weight)| {
                let ceil = accum + *weight;
                *weight = ceil;
                ceil
            });

            /* Choose a random number and find the first merge whose weight is greater */
            let random_number = rand::thread_rng().gen_range(0..total_weight);
            let merge = possible_merges
                .iter()
                .find(|(_, cum)| *cum > random_number)
                .unwrap()
                .0;

            /* Perform the merge */
            Ok(self.try_merge_cages(*merge.0, *merge.1))
        }
    }

    fn merge_random_cages(&mut self) -> bool {
        match self.try_merge_random_cages() {
            Ok(true) => return true,
            Err(()) => return false,
            _ => self.merge_random_cages(),
        }
    }

    fn try_eliminate_cage(&mut self) -> Result<bool, ()> {
        if self.merge_random_cages() {
            /* After a cage has been removed, try to solve */
            let puzzle = Puzzle::from_serializable(self.serialize());
            match puzzle.solve() {
                Ok(solutions) => Ok(solutions.len() == 1),
                Err(()) => panic!("The puzzle should not have contradictions"),
            }
        } else {
            Err(())
        }
    }

    pub fn eliminate_cage(&mut self) -> bool {
        let mut copy = self.clone();
        match copy.try_eliminate_cage() {
            Ok(true) => {
                /* Success */
                *self = copy;
                true
            }
            Ok(false) => {
                /* Copy updated adjacent cage state */
                self.cages.iter_mut().for_each(|(cage_index, cage)| {
                    if let Some(other_cage) = copy.cages.get(cage_index) {
                        cage.adjacent_cages = other_cage.adjacent_cages.clone();
                    } else {
                        cage.adjacent_cages.clear();
                    }
                });
                self.eliminate_cage()
            }
            Err(()) => false,
        }
    }

    // pub fn eliminate_cage(&self) -> Option<Self> {
    //     let mut output = self.clone();
    // }

    // fn merge_cage(&mut self, old_cage_index: usize, new_cage_index: usize) -> bool {}

    // pub fn eliminate_cage(&self) -> Self {
    //     let mut output = self.clone();
    //     loop {
    //         match output.remaining_merges.pop() {
    //             Some((index_a, index_b)) => {
    //                 if output.cages[index_a] == output.cages[index_b] {
    //                     /* These cells already belong to the same cage */
    //                 } else {
    //                     /* Rename one of the cages */
    //                     let mut copy = output.clone();
    //                     let old_cage_index = copy.cages[index_a];
    //                     let new_cage_index = copy.cages[index_b];
    //                     let merged_cage_values = copy
    //                         .cages
    //                         .iter_mut()
    //                         .enumerate()
    //                         .filter_map(|(i, cage)| {
    //                             if *cage == old_cage_index {
    //                                 *cage = new_cage_index
    //                             }
    //                             (*cage == new_cage_index).then_some(self.numbers[i])
    //                         })
    //                         .collect::<Vec<usize>>();

    //                     /* Confirm that the new cage contains unique values */
    //                     if merged_cage_values.len() < 6
    //                         && merged_cage_values.len()
    //                             == BTreeSet::from_iter(merged_cage_values.iter()).len()
    //                     {
    //                         /* Attempt to solve the puzzle */
    //                         let puzzle = Puzzle::from_serializable(self.serialize());
    //                         match puzzle.solve() {
    //                             Ok(solutions) => {
    //                                 if solutions.len() == 1 {
    //                                     output = copy;
    //                                     break;
    //                                 }
    //                             }
    //                             Err(()) => {
    //                                 println!("Error");
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             None => break,
    //         }
    //     }
    //     output
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::Generator;

    #[test]
    fn merge_cages() {
        let mut gen = Generator::new_canonical();
        assert!(gen.try_merge_cages(0, 1));
        assert!(!gen.cages.contains_key(&1));
        assert_eq!(gen.cages[&0].cells, BTreeSet::from([0, 1]));
        assert_eq!(gen.cages[&0].adjacent_cages, BTreeSet::from([2, 9, 10]));
        assert_eq!(gen.cages[&2].adjacent_cages, BTreeSet::from([0, 3, 11]));
        assert_eq!(gen.cages[&9].adjacent_cages, BTreeSet::from([0, 10, 18]));
        assert_eq!(
            gen.cages[&10].adjacent_cages,
            BTreeSet::from([0, 9, 11, 19])
        );
    }

    #[test]
    fn merge_cages_unsuccessful() {
        let mut gen = Generator::new_canonical();
        assert!(gen.try_merge_cages(0, 1));
        assert!(gen.try_merge_cages(0, 2));
        assert!(gen.try_merge_cages(0, 3));
        assert!(!gen.try_merge_cages(0, 9));
    }

    #[test]
    fn merge_random_cages() {
        let mut gen = Generator::new_canonical();
        assert_eq!(gen.try_merge_random_cages(), Ok(true));
        assert_eq!(gen.cages.len(), 80);
        assert_eq!(
            gen.cages
                .values()
                .filter(|cage| cage.cells.len() == 2)
                .count(),
            1
        );
    }

    #[test]
    fn merge_random_cages_loop() {
        let mut gen = Generator::new_canonical();
        assert_eq!(gen.merge_random_cages(), true);
        assert_eq!(gen.cages.len(), 80);
        assert_eq!(
            gen.cages
                .values()
                .filter(|cage| cage.cells.len() == 2)
                .count(),
            1
        );
    }
}
