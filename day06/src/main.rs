use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let coords = read_coords(&input)?;

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Coord {
    x: u32,
    y: u32,
}

impl FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(?P<x>),\s*(?P<y>)$").unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid coordinate: {}", s))?;

        Ok(Coord { x: caps["x"].parse()?, y: caps["y"].parse()? })
    }
}

fn read_coords(input: &str) -> Result<Vec<Coord>> {
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort_unstable();
    lines.iter().map(|line| line.parse()).collect()
}
