// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::{
    collections::{BTreeMap, BTreeSet},
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

    pub fn to_svg_file<P: AsRef<Path>>(
        &self,
        output_path: P,
        title: &str,
    ) -> Result<(), Box<dyn Error>> {
        use chrono::prelude::*;
        use svg::node::{
            element::{Group, Line, Rectangle, Text},
            Text as TextNode,
        };
        use svg::Document;

        const CELL_SIZE: u32 = 100;
        const MARGIN: u32 = 80;
        const HEADER_HEIGHT: u32 = 160;
        const TOTAL_WIDTH: u32 = CELL_SIZE * 9 + MARGIN * 2;
        const TOTAL_HEIGHT: u32 = CELL_SIZE * 9 + HEADER_HEIGHT + MARGIN;

        /* Calculate cage adjacency */
        let cage_indices_by_cell =
            self.cages
                .iter()
                .enumerate()
                .fold([0; 81], |mut accum, (cage_index, cage)| {
                    for cell_index in &cage.cell_indices {
                        accum[*cell_index] = cage_index;
                    }
                    accum
                });
        let mut interference_map: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); self.cages.len()];
        let mut insert_interference = |a: usize, b: usize| {
            let cage_a = cage_indices_by_cell[a];
            let cage_b = cage_indices_by_cell[b];
            if cage_a != cage_b {
                interference_map[cage_a].insert(cage_b);
                interference_map[cage_b].insert(cage_a);
            }
        };
        for row in 0..9 {
            for col in 0..8 {
                insert_interference(row * 9 + col, row * 9 + col + 1);
                insert_interference(col * 9 + row, (col + 1) * 9 + row);
            }
        }
        for row in 0..8 {
            for col in 0..8 {
                insert_interference(row * 9 + col, (row + 1) * 9 + col + 1);
                insert_interference((row + 1) * 9 + col, row * 9 + col + 1);
            }
        }
        let mut interference_map = interference_map.into_iter().collect::<Vec<_>>();

        /* Initially give each cage its own color, then reduce colors */
        const COLORS: [&str; 9] = [
            "#d0d0ff", "#d0ffd0", "#ffd0d0", "#ffffd0", "#ffd0ff", "#d0ffff", "#e0e0d0", "#e0d0e0",
            "#d0e0e0",
        ];
        let mut color_map = (0..interference_map.len()).collect::<Vec<usize>>();
        let mut color_population = self
            .cages
            .iter()
            .map(|cage| cage.cell_indices.len())
            .collect::<Vec<usize>>();
        let mut progress = true;
        while progress && color_population.iter().filter(|p| **p > 0).count() > COLORS.len() {
            progress = false;
            for (cage_index, (interference_set, cage)) in
                interference_map.iter().zip(&self.cages).enumerate()
            {
                let old_color = color_map[cage_index];
                let old_population = color_population[old_color];
                let interfering_colors = interference_set
                    .iter()
                    .map(|other_cage_index| color_map[*other_cage_index])
                    .collect::<BTreeSet<usize>>();
                if let Some((new_color, _)) = color_population
                    .iter()
                    .enumerate()
                    .filter(|(color_index, population)| {
                        *color_index != old_color
                            && **population > 0
                            && !interfering_colors.contains(color_index)
                    })
                    .min_by_key(|(i, p)| **p)
                {
                    assert!(old_color != new_color);
                    progress = true;
                    color_population[new_color] += cage.cell_indices.len();
                    color_population[old_color] -= cage.cell_indices.len();
                    color_map[cage_index] = new_color;
                }
            }
        }

        /* Remap colors to the desired range */
        let remapped_colors = color_population.iter().enumerate().fold(
            BTreeMap::new(),
            |mut accum, (color_index, population)| {
                if *population > 0 {
                    accum.insert(color_index, accum.len());
                }
                accum
            },
        );

        /* White background */
        let background = Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", TOTAL_WIDTH)
            .set("height", TOTAL_HEIGHT)
            .set("stroke", "none")
            .set("fill", "white");

        /* Generate colored squares */
        let mut squares = vec![];
        for row in 0..9 {
            for col in 0..9 {
                squares.push(
                    Rectangle::new()
                        .set("x", MARGIN + CELL_SIZE * row)
                        .set("y", HEADER_HEIGHT + CELL_SIZE * col)
                        .set("width", CELL_SIZE)
                        .set("height", CELL_SIZE)
                        .set("stroke", "none")
                        .set(
                            "fill",
                            COLORS[remapped_colors
                                [&color_map[cage_indices_by_cell[(row * 9 + col) as usize]]]],
                        ),
                );
            }
        }
        let puzzle_group = squares.into_iter().fold(Group::new(), |g, s| g.add(s));

        /* Generate cage indices */
        let text_group = self
            .cages
            .iter()
            .enumerate()
            .map(|(i, cage)| {
                let col = cage
                    .cell_indices
                    .iter()
                    .map(|cell_index| *cell_index % 9)
                    .min()
                    .unwrap();
                let row = cage
                    .cell_indices
                    .iter()
                    .filter(|cell_index| *cell_index % 9 == col)
                    .map(|cell_index| *cell_index / 9)
                    .min()
                    .unwrap();
                Text::new()
                    .set("x", MARGIN + CELL_SIZE * row as u32 + 6)
                    .set("y", HEADER_HEIGHT + CELL_SIZE * col as u32 + 24)
                    .set("font-size", 24)
                    .add(TextNode::new(format!("{}", cage.sum)))
            })
            .fold(Group::new(), |g, t| g.add(t));

        /* Generate lines */
        let horizontal_line_group = (0..=9)
            .map(|i| {
                let y = HEADER_HEIGHT + CELL_SIZE * i;
                Line::new()
                    .set("x1", MARGIN - 2)
                    .set("x2", TOTAL_WIDTH - MARGIN + 2)
                    .set("y1", y)
                    .set("y2", y)
                    .set("stroke", "black")
                    .set("stroke-width", if i % 3 == 0 { 4 } else { 1 })
            })
            .fold(Group::new(), |g, l| g.add(l));
        let vertical_line_group = (0..=9)
            .map(|i| {
                let x = MARGIN + CELL_SIZE * i;
                Line::new()
                    .set("y1", HEADER_HEIGHT - 2)
                    .set("y2", TOTAL_HEIGHT - MARGIN + 2)
                    .set("x1", x)
                    .set("x2", x)
                    .set("stroke", "black")
                    .set("stroke-width", if i % 3 == 0 { 4 } else { 1 })
            })
            .fold(Group::new(), |g, l| g.add(l));

        /* Generate titles */
        let title = Text::new()
            .set("text-anchor", "middle")
            .set("x", TOTAL_WIDTH / 2)
            .set("y", HEADER_HEIGHT - 48)
            .set("font-size", 60)
            .add(TextNode::new(title));
        let subtitle = Text::new()
            .set("text-anchor", "middle")
            .set("x", TOTAL_WIDTH / 2)
            .set("y", HEADER_HEIGHT - 16)
            .set("font-size", 24)
            .add(TextNode::new(format!("{} Cages", self.cages.len())));
        let date = Text::new()
            .set("x", MARGIN)
            .set("y", TOTAL_HEIGHT - MARGIN + 24)
            .set("font-size", 20)
            .add(TextNode::new(Local::now().format("%B %e, %Y").to_string()));
        let copyright = Text::new()
            .set("text-anchor", "end")
            .set("x", TOTAL_WIDTH - MARGIN)
            .set("y", TOTAL_HEIGHT - MARGIN + 24)
            .set("font-size", 20)
            .add(TextNode::new(format!(
                "Copyright {} by Daniel Winkelman.",
                Local::now().format("%Y")
            )));

        let document = Document::new()
            .set("viewBox", (0, 0, TOTAL_WIDTH, TOTAL_HEIGHT))
            .add(background)
            .add(puzzle_group)
            .add(text_group)
            .add(horizontal_line_group)
            .add(vertical_line_group)
            .add(title)
            .add(subtitle)
            .add(date)
            .add(copyright);

        svg::save(output_path, &document)?;
        Ok(())
    }
}
