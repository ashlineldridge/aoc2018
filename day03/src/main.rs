use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, Read},
    str::FromStr,
};

type ClaimGrid = HashMap<(u32, u32), u32>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let claims = parse_claims(&input)?;
    let grid = claim_grid(&claims);
    let overlap = grid_overlap(&grid, 2);
    println!("Total square inches of claim overlap is {}", overlap);

    let unique_claims = unique_claims(&claims, &grid);
    println!("Found unique claims: {:?}", unique_claims);

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Claim {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct IterPoints<'a> {
    claim: &'a Claim,
    px: u32,
    py: u32,
}

impl<'a> Iterator for IterPoints<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.py >= (self.claim.y + self.claim.height) {
            return None;
        }

        let res = Some((self.px, self.py));

        self.px += 1;
        if self.px >= (self.claim.x + self.claim.width) {
            self.px = self.claim.x;
            self.py += 1;
        }

        res
    }
}

impl Claim {
    fn iter_points(&self) -> IterPoints {
        IterPoints {
            claim: self,
            px: self.x,
            py: self.y,
        }
    }
}

impl FromStr for Claim {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)^
                \#(?P<id>\d+)\s+
                @\s+
                (?P<x>\d+),(?P<y>\d+):\s+
                (?P<width>\d+)x(?P<height>\d+)$
            "
            )
            .unwrap();
        }
        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid claim entry: {}", s))?;
        Ok(Claim {
            id: caps["id"].parse()?,
            x: caps["x"].parse()?,
            y: caps["y"].parse()?,
            width: caps["width"].parse()?,
            height: caps["height"].parse()?,
        })
    }
}

fn parse_claims(input: &str) -> Result<Vec<Claim>> {
    input.lines().map(|line| line.parse()).collect()
}

fn claim_grid(claims: &[Claim]) -> ClaimGrid {
    let mut grid = ClaimGrid::new();
    for claim in claims {
        for (x, y) in claim.iter_points() {
            *grid.entry((x, y)).or_default() += 1;
        }
    }

    grid
}

fn grid_overlap(grid: &ClaimGrid, min: u32) -> u32 {
    grid.values().filter(|&&count| count >= min).count() as u32
}

fn unique_claims(claims: &[Claim], grid: &ClaimGrid) -> Vec<Claim> {
    let mut unique_claims = vec![];
    for claim in claims {
        if is_unique(claim, grid) {
            unique_claims.push(claim.clone());
        }
    }

    unique_claims
}

fn is_unique(claim: &Claim, grid: &ClaimGrid) -> bool {
    claim.iter_points().all(|p| grid[&p] == 1)
}
