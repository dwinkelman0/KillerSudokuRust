// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::ks::puzzle::Puzzle;

pub struct Cage {
    pub puzzle: Rc<RefCell<Puzzle>>,
    pub cells: Vec<usize>,
    pub sum: usize,
}

impl Cage {
    pub fn get_possible_sums(&self) -> u64 {
        self.cells.iter().fold(1, |accum, cell| {
            self.puzzle.borrow().board[*cell].fold_possible_sums(accum)
        })
    }
}

impl Display for Cage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cells)
    }
}
