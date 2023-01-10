// Copyright 2022 by Daniel Winkelman. All rights reserved.

mod ks;

use ks::generator::Generator;

fn main() {
    let mut gen = Generator::new_random();
    while gen.eliminate_cage() {
        gen.serialize().to_json_file("output.json").unwrap();
        println!("{} cages", gen.get_num_cages());
    }
    gen.serialize().to_svg_file("output.svg", "demo").unwrap();
}

#[cfg(test)]
mod tests {
    macro_rules! solve_from_file {
        ($test_name:ident, $filename:literal) => {
            mod $test_name {
                use crate::ks::{io::SerializablePuzzle, puzzle::Puzzle};
                #[test]
                fn solve_from_file() {
                    let serialized_puzzle =
                        SerializablePuzzle::from_str(include_str!($filename)).unwrap();
                    let puzzle = Puzzle::from_serializable(serialized_puzzle);
                    let solutions = puzzle.solve();
                    assert!(solutions.is_ok());
                    if let Ok(solutions) = solutions {
                        assert_eq!(solutions.len(), 1);
                    }
                }
            }
        };
    }

    solve_from_file! {basic, "ks/test/puzzle_0.json"}
}
