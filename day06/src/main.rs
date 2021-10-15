#![feature(int_abs_diff)]

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    cmp,
    collections::{HashMap, HashSet},
    io::{self, Read},
    iter,
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let markers = read_markers(&input)?;

    let answer = part1(&markers)?;
    println!("Answer to part 1: {}", answer);

    let answer = part2(&markers, 10000)?;
    println!("Answer to part 2: {}", answer);

    Ok(())
}

fn part1(markers: &HashSet<Coord>) -> Result<u32> {
    let closest_grid = build_closest_grid(markers);
    let mut count_by_marker: HashMap<&Coord, u32> = markers.iter().zip(iter::repeat(0)).collect();

    for coord in closest_grid.grid_iter() {
        if let Some(marker) = closest_grid.get(coord) {
            if coord.x == 0 || coord.y == 0 || coord.x == closest_grid.width - 1 || coord.y == closest_grid.height - 1 {
                count_by_marker.remove(marker);
            } else {
                count_by_marker.entry(marker).and_modify(|c| *c += 1);
            }
        }
    }

    Ok(*count_by_marker.values().max().unwrap_or(&0))
}

fn part2(markers: &HashSet<Coord>, max_dist: u32) -> Result<u32> {
    let proximity_grid = build_proximity_grid(markers, max_dist);
    Ok(proximity_grid.coords.values().filter(|v| **v).count() as u32)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coord {
    x: u32,
    y: u32,
}

impl Coord {
    fn dist(&self, other: &Coord) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

impl Default for Coord {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(?P<x>\d+),\s*(?P<y>\d+)$").unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid coordinate: {}", s))?;

        Ok(Coord {
            x: caps["x"].parse()?,
            y: caps["y"].parse()?,
        })
    }
}

struct Grid<T> {
    width: u32,
    height: u32,
    coords: HashMap<Coord, T>,
}

impl<T> Grid<T> {
    fn build<F>(width: u32, height: u32, f: F) -> Grid<T>
    where
        F: Fn(Coord) -> Option<T>,
    {
        let mut coords = HashMap::new();
        for coord in GridIter::new(width, height) {
            if let Some(t) = f(coord) {
                coords.insert(coord, t);
            }
        }

        Grid {
            width,
            height,
            coords,
        }
    }

    fn get(&self, coord: Coord) -> Option<&T> {
        self.coords.get(&coord)
    }

    fn grid_iter(&self) -> GridIter {
        GridIter::new(self.width, self.height)
    }
}

struct GridIter {
    width: u32,
    height: u32,
    cur: Coord,
}

impl GridIter {
    fn new(width: u32, height: u32) -> GridIter {
        GridIter {
            width,
            height,
            cur: Default::default(),
        }
    }
}

impl Iterator for GridIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.y >= (self.height) {
            return None;
        }

        let next = self.cur;

        self.cur.x += 1;
        if self.cur.x >= (self.width) {
            self.cur.x = 0;
            self.cur.y += 1;
        }

        Some(next)
    }
}

fn read_markers(input: &str) -> Result<HashSet<Coord>> {
    input.lines().map(|line| line.parse()).collect()
}

fn grid_size(coords: &HashSet<Coord>) -> (u32, u32) {
    let mut x_max = 0;
    let mut y_max = 0;
    for coord in coords {
        x_max = cmp::max(coord.x, x_max);
        y_max = cmp::max(coord.y, y_max);
    }

    (x_max + 1, y_max + 1)
}

fn build_marker_grid(markers: &HashSet<Coord>) -> Grid<bool> {
    let (width, height) = grid_size(&markers);
    Grid::build(width, height, |coord| {
        if markers.contains(&coord) {
            Some(true)
        } else {
            None
        }
    })
}

fn build_closest_grid(markers: &HashSet<Coord>) -> Grid<Coord> {
    let marker_grid = build_marker_grid(markers);
    Grid::build(marker_grid.width, marker_grid.height, |coord| {
        let mut exclusive = true;
        let mut closest_marker = None;
        let mut closest_dist = 0;

        for marker in markers {
            let dist = coord.dist(marker);
            if closest_marker.is_some() {
                if dist < closest_dist {
                    closest_marker = Some(*marker);
                    closest_dist = dist;
                    exclusive = true;
                } else if dist == closest_dist {
                    exclusive = false;
                }
            } else {
                closest_marker = Some(*marker);
                closest_dist = dist;
            }
        }

        if exclusive {
            closest_marker
        } else {
            None
        }
    })
}

fn build_proximity_grid(markers: &HashSet<Coord>, max_dist: u32) -> Grid<bool> {
    let marker_grid = build_marker_grid(markers);
    Grid::build(marker_grid.width, marker_grid.height, |coord| {
        let dist = markers
            .iter()
            .fold(0, |acc, b| acc + b.dist(&coord));
        Some(dist < max_dist)
    })
}
