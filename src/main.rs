// Copyright 2022 by Daniel Winkelman. All rights reserved.

mod ks;

use crate::ks::puzzle::Puzzle;

fn main() {
    let p = Puzzle::new();
    println!("{}", p.borrow());
}
