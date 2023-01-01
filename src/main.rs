// Copyright 2022 by Daniel Winkelman. All rights reserved.

mod ks;

use crate::ks::puzzle::Puzzle;

fn main() {
    let mut p = Puzzle::new();
    p.init_cages(vec![
        (23, vec![0, 1, 2, 10, 11]),
        (17, vec![3, 4, 5]),
        (7, vec![6, 7]),
        (7, vec![12, 13]),
        (17, vec![14, 15, 24]),
        (12, vec![16, 25, 34]),
        (9, vec![19, 20]),
        (21, vec![21, 22, 23, 30]),
        (22, vec![8, 17, 26, 35]),
        (10, vec![28, 29]),
        (17, vec![31, 32, 40, 41]),
        (10, vec![33, 42, 51]),
        (26, vec![9, 18, 27, 36, 45]),
        (20, vec![37, 46, 55, 64]),
        (16, vec![38, 39]),
        (17, vec![43, 44, 53]),
        (9, vec![47, 48]),
        (13, vec![49, 50]),
        (13, vec![52, 61, 70]),
        (11, vec![54, 63]),
        (16, vec![56, 65, 74]),
        (17, vec![57, 66]),
        (8, vec![58, 59]),
        (11, vec![60, 68, 69]),
        (11, vec![62, 71]),
        (16, vec![67, 75, 76]),
        (8, vec![72, 73]),
        (21, vec![77, 78, 79, 80]),
    ]);
    match p.solve() {
        Ok(solutions) => {
            match solutions.len() {
                0 => println!("Found no solutions, possibly recursion limit was reached?"),
                1 => {
                    println!("Found one solution:");
                    println!("{}", solutions.first().unwrap())
                }
                _ => println!("Found {} solutions", solutions.len()),
            }
            println!("Found {} solution(s)", solutions.len())
        }
        Err(()) => println!("Contradiction!"),
    }
}
