use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::cell::RefCell;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
    str::FromStr,
};

type ClaimGrid = HashMap<(u32, u32), RefCell<HashSet<Claim>>>;

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
    input.lines().map(|line| Claim::from_str(line)).collect()
}

fn claim_grid(claims: &[Claim]) -> ClaimGrid {
    let mut grid: ClaimGrid = HashMap::new();
    for claim in claims {
        for x in claim.x..claim.x + claim.width {
            for y in claim.y..claim.y + claim.height {
                let mut claims = grid.entry((x, y)).or_default().borrow_mut();
                claims.insert(claim.clone());
            }
        }
    }

    grid
}

fn grid_overlap(grid: &ClaimGrid, min: usize) -> u32 {
    grid.values().fold(0, |acc, c| {
        if c.borrow().len() >= min {
            acc + 1
        } else {
            acc
        }
    })
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
    for x in claim.x..claim.x + claim.width {
        for y in claim.y..claim.y + claim.height {
            match grid.get(&(x, y)).map(|r| r.borrow()) {
                None => return false,
                Some(claims) => {
                    if claims.len() != 1 || !claims.contains(claim) {
                        return false;
                    }
                }
            }
        }
    }

    true
}
