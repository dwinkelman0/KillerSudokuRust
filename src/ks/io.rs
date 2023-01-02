// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SerializableCage {
    pub sum: usize,
    pub cell_indices: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct SerializablePuzzle {
    pub cell_values: Vec<usize>,
    pub cages: Vec<SerializableCage>,
}

#[allow(unused)]
impl SerializablePuzzle {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
    }

    pub fn from_str(data: &str) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_str(data)?)
    }

    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        Ok(serde_json::to_writer_pretty(
            BufWriter::new(File::create(path)?),
            self,
        )?)
    }

    pub fn to_str(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}
