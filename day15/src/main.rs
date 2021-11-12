use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{self, Read};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let game: Game = input.parse()?;

    Ok(())
}

enum Piece {
    Elf { health: u32, attack_power: u32 },
    Goblin { health: u32, attack_power: u32 },
    Wall,
    Open,
}

impl FromStr for Piece {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#" => Ok(Piece::Wall),
            "." => Ok(Piece::Open),
            "E" => Ok(Piece::new_elf()),
            "G" => Ok(Piece::new_goblin()),
            _ => bail!("Unknown piece: {}", s),
        }
    }
}

impl Piece {
    const DEFAULT_HEALTH: u32 = 200;
    const DEFAULT_ATTACK_POWER: u32 = 3;

    fn new_elf() -> Self {
        Self::Elf {
            health: Piece::DEFAULT_HEALTH,
            attack_power: Piece::DEFAULT_ATTACK_POWER,
        }
    }

    fn new_goblin() -> Self {
        Self::Goblin {
            health: Piece::DEFAULT_HEALTH,
            attack_power: Piece::DEFAULT_ATTACK_POWER,
        }
    }
}

struct Game {
    grid: HashMap<Coord, Piece>,
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = HashMap::new();
        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let piece = ch.to_string().parse()?;
                grid.insert(Coord::new(x, y), piece);
            }
        }

        Ok(Game { grid })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Coord {
        Coord { x, y }
    }
}

// Iterate over each unit in reading order. On each's turn:
// 1. Identify all possible targets. If none, game is over.
// 2. Idenitfy all free/open squares in range of each target.
// 4. If the unit is not already in range, it moves one step towards the closest free square.
// 5. If the unit is currently in range it attacks.
