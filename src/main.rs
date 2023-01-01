// Copyright 2022 by Daniel Winkelman. All rights reserved.

mod ks;

use ks::{io::SerializablePuzzle, puzzle::Puzzle};

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_from_file() {
        let serialized_puzzle =
            SerializablePuzzle::from_str(include_str!("ks/test/puzzle_0.json")).unwrap();
        let puzzle = Puzzle::from_serializable(serialized_puzzle);
        let solutions = puzzle.solve();
        assert!(solutions.is_ok());
        if let Ok(solutions) = solutions {
            assert_eq!(solutions.len(), 1);
        }
    }
}
