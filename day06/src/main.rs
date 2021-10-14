#![feature(int_abs_diff)]

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let answer = part1(&input)?;
    println!("Answer to part 1: {}", answer);

    let answer = part2(&input, 10000)?;
    println!("Answer to part 2: {}", answer);

    Ok(())
}

fn part1(input: &str) -> Result<u32> {
    let mut beacons = read_beacons(input)?;
    let (width, height) = grid_size(&beacons);
    let grid = build_grid(&beacons, width, height);

    for x in 0..width {
        for y in 0..height {
            if let Some(closest) = grid.get(&Coord { x, y }) {
                if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    beacons.remove(closest);
                }
            }
        }
    }

    let mut counts: HashMap<&Coord, u32> = HashMap::new();
    for beacon in &beacons {
        let count = grid.values().filter(|&v| v == beacon).count();
        counts.insert(beacon, count as u32);
    }

    Ok(*counts.values().max().unwrap_or(&0))
}

fn part2(input: &str, within_dist: u32) -> Result<u32> {
    let beacons = read_beacons(input)?;
    let (width, height) = grid_size(&beacons);
    let mut region_size = 0;

    for x in 0..width {
        for y in 0..height {
            let dist = beacons
                .iter()
                .fold(0, |acc, b| acc + b.dist(&Coord { x, y }));
            if dist < within_dist {
                region_size += 1;
            }
        }
    }

    Ok(region_size)
}

type Grid = HashMap<Coord, Coord>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Coord {
    x: u32,
    y: u32,
}

impl Coord {
    fn dist(&self, other: &Coord) -> u32 { self.x.abs_diff(other.x) + self.y.abs_diff(other.y) }
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

fn read_beacons(input: &str) -> Result<HashSet<Coord>> {
    input.lines().map(|line| line.parse()).collect()
}

fn grid_size(beacons: &HashSet<Coord>) -> (u32, u32) {
    let mut x_max = 0;
    let mut y_max = 0;
    for c in beacons {
        if c.x > x_max {
            x_max = c.x;
        }
        if c.y > y_max {
            y_max = c.y;
        }
    }
    (x_max + 1, y_max + 1)
}

fn build_grid(beacons: &HashSet<Coord>, width: u32, height: u32) -> Grid {
    let mut grid = Grid::new();
    for x in 0..width {
        for y in 0..height {
            let mut closest_beacons = vec![];
            let mut closest_dist = 0;
            for beacon in beacons {
                let dist = beacon.dist(&Coord { x, y });
                if dist == 0 {
                    // No beacon can be closer so stop searching.
                    closest_beacons = vec![beacon];
                    break;
                }

                if closest_beacons.is_empty() || dist < closest_dist {
                    closest_dist = dist;
                    closest_beacons = vec![beacon];
                } else if dist == closest_dist {
                    closest_beacons.push(beacon);
                }
            }

            if closest_beacons.len() == 1 {
                grid.insert(Coord { x, y }, closest_beacons.pop().unwrap().clone());
            }
        }
    }

    grid
}
