use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Write};
use std::io::{self, Read};
use std::path;
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // let input = std::fs::read_to_string("day15/input/sample.txt")?;

    let game: Game = input.parse()?;
    let paths = game.shortest_paths(Coord::new(1, 1));

    println!("{}", game);
    println!("Calculated {} paths", paths.len());

    dbg!(paths.get(&Coord::new(6, 7)));

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Piece {
    Combatant(Combatant),
    Wall,
    Empty,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Combatant {
    kind: CombatantKind,
    health: u32,
    attack_power: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CombatantKind {
    Elf,
    Goblin,
}

impl Combatant {
    fn is_enemy(&self, other: &Combatant) -> bool {
        self.kind != other.kind
    }
}

impl Piece {
    const DEFAULT_HEALTH: u32 = 200;
    const DEFAULT_ATTACK_POWER: u32 = 3;

    fn new_elf() -> Self {
        Self::Combatant(
            Combatant {
                kind: CombatantKind::Elf,
                health: Piece::DEFAULT_HEALTH,
                attack_power: Piece::DEFAULT_ATTACK_POWER,
            }
        )
    }

    fn new_goblin() -> Self {
        Self::Combatant(
            Combatant {
                kind: CombatantKind::Goblin,
                health: Piece::DEFAULT_HEALTH,
                attack_power: Piece::DEFAULT_ATTACK_POWER,
            }
        )
    }

    fn is_combatant(&self) -> bool {
        matches!(self, Piece::Combatant { .. })
    }

    fn is_enemy(&self, other: &Piece) -> bool {
        match (self, other) {
            (Piece::Combatant(c1), Piece::Combatant(c2)) => c1.is_enemy(c2),
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        *self == Piece::Empty
    }
}

impl FromStr for Piece {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "E" => Ok(Piece::new_elf()),
            "G" => Ok(Piece::new_goblin()),
            "#" => Ok(Piece::Wall),
            "." => Ok(Piece::Empty),
            _ => bail!("Unknown piece: {}", s),
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Piece::Combatant(c) if c.kind == CombatantKind::Elf => 'E',
            Piece::Combatant(_)  => 'G',
            Piece::Wall => '#',
            Piece::Empty => '.',
        };

        f.write_char(ch)
    }
}

struct Game {
    grid: HashMap<Coord, Piece>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }

    fn neighbors(&self) -> HashSet<Coord> {
        vec![
            Coord::new(self.x, self.y - 1), // Above.
            Coord::new(self.x, self.y + 1), // Below.
            Coord::new(self.x - 1, self.y), // Left.
            Coord::new(self.x + 1, self.y), // Right.
        ].into_iter().collect::<HashSet<_>>()
    }
}

impl Ord for Coord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.y.cmp(&other.y) {
            std::cmp::Ordering::Equal => self.x.cmp(&other.x),
            v => v,
        }
    }
}

impl PartialOrd for Coord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = HashMap::new();
        let mut x_max = 0;
        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let piece = ch.to_string().parse()?;
                grid.insert(Coord::new(x as i32, y as i32), piece);
                x_max = x_max.max(x);
            }
        }

        Ok(Game {
            grid,
            width: x_max + 1,
            height: s.lines().count(),
        })
    }
}

impl Game {
    fn iter(&self) -> GameIter {
        GameIter {
            curr: Coord::new(0, 0),
            width: self.width,
            height: self.height,
        }
    }

    fn try_attack(&mut self, combatant: &Combatant, coord: &Coord) -> bool {
        if let Some((mut enemy, enemy_coord)) = self.in_range(combatant, &coord) {
            enemy.health = enemy.health.saturating_sub(combatant.attack_power);
            if enemy.health == 0 {
                self.grid.insert(enemy_coord, Piece::Empty);
            }

            true
        } else {
            false
        }
    }

    fn play_round(&mut self) -> Result<()> {
        for coord1 in self.iter() {
            let piece1 = self.grid[&coord1];
            if let Piece::Combatant(c) = piece1 {
                if self.try_attack(&c, &coord1) {
                    continue;
                }
            } else {
                continue;
            }

            if !piece1.is_combatant() {
                continue;
            }

            let mut destinations = vec![];
            for coord2 in self.iter() {
                let piece2 = self.grid[&coord2];
                if !piece2.is_enemy(&piece1) {
                    continue;
                }

                let mut neighbors = self
                    .get_free_neighbors(coord2)
                    .into_iter()
                    .collect::<Vec<_>>();
                destinations.append(&mut neighbors);
            }

            let shortest_paths = self.shortest_paths(coord1);
            let mut closest_destination = None;
            let mut closest_distance = 0;
            for destination in destinations {
                let path = &shortest_paths[&destination];
                match closest_destination {
                    None | Some(_) if path.len() < closest_distance => {
                        closest_destination = Some(destination);
                        closest_distance = path.len();
                    }
                    Some(d) if path.len() == closest_distance && destination < d => {
                        closest_destination = Some(destination);
                        closest_distance = path.len();
                    }
                    _ => (),
                };
            }

            if let Some(closest_destination) = closest_destination {
                let path = &shortest_paths[&closest_destination];
                self.grid.insert(path[0], piece1);
                self.grid.insert(coord1, Piece::Empty);

                self.try_attack(&c, &path[0]);
            }
        }

        Ok(())
    }

    fn get_free_neighbors(&self, coord: Coord) -> HashSet<Coord> {
        coord
            .neighbors()
            .into_iter()
            .filter(|coord| matches!(self.grid.get(coord), Some(Piece::Empty)))
            .collect::<HashSet<_>>()
    }

    fn in_range(&self, combatant: &Combatant, coord: &Coord) -> Option<(Combatant, Coord)> {
        let mut in_range = coord
            .neighbors()
            .into_iter()
            .filter_map(|coord| {
                match self.grid.get(&coord) {
                    Some(p @ Piece::Combatant(c)) if combatant.is_enemy(c) => Some((*c, coord)),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        in_range.sort_unstable_by(|a, b| {
            match a.0.health.cmp(&b.0.health) {
                std::cmp::Ordering::Equal => a.1.cmp(&b.1),
                v => v,
            }
        });

        in_range.first().cloned()
    }

    fn get_all_empty(&self) -> HashSet<Coord> {
        self.grid
            .iter()
            .filter_map(
                |(coord, piece)| {
                    if piece.is_empty() {
                        Some(*coord)
                    } else {
                        None
                    }
                },
            )
            .collect::<HashSet<_>>()
    }

    fn shortest_paths(&self, start: Coord) -> HashMap<Coord, Vec<Coord>> {
        #[derive(Clone, Debug)]
        struct Path {
            coords: Vec<Coord>,
            cost: u32,
        }

        let mut paths = HashMap::new();
        paths.insert(
            start,
            Path {
                coords: vec![],
                cost: 0,
            },
        );

        let mut unvisited = self.get_all_empty();
        for coord in &unvisited {
            paths.insert(
                *coord,
                Path {
                    coords: vec![],
                    cost: u32::MAX,
                },
            );
        }

        let mut current = start;
        while !unvisited.is_empty() {
            let path_to_current = paths[&current].clone();
            let neighbors = self
                .get_free_neighbors(current)
                .intersection(&unvisited)
                .cloned()
                .collect::<Vec<Coord>>();

            let mut next_paths = vec![];
            for neighbor in &neighbors {
                let cost = path_to_current.cost + 1;
                let mut path_to_neighbor = paths[neighbor].clone();
                if cost < path_to_neighbor.cost {
                    let mut coords = path_to_current.coords.clone();
                    coords.push(*neighbor);

                    path_to_neighbor = Path { coords, cost };
                    paths.insert(*neighbor, path_to_neighbor.clone());
                }

                next_paths.push(path_to_neighbor);
            }

            next_paths.sort_unstable_by(|a, b| {
                if a.cost != b.cost {
                    a.cost.cmp(&b.cost)
                } else {
                    a.coords.last().unwrap().cmp(b.coords.last().unwrap())
                }
            });

            if let Some(next) = next_paths.first() {
                unvisited.remove(&current);
                current = *next.coords.last().unwrap();
            } else {
                // We are finished.
                break;
            }
        }

        paths
            .into_iter()
            .map(|(coord, path)| (coord, path.coords))
            .collect::<HashMap<_, _>>()
    }
}

struct GameIter {
    width: usize,
    height: usize,
    curr: Coord,
}

impl Iterator for GameIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.y >= self.height as i32 {
            None
        } else {
            let coord = self.curr;

            self.curr.x += 1;
            if self.curr.x >= self.width as i32 {
                self.curr.x = 0;
                self.curr.y += 1;
            }

            Some(coord)
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("  ");
        for x in 0..self.width {
            buf += format!("{}", x % 10).as_str();
        }
        buf += "\n";

        for y in 0..self.height {
            buf += format!("{} ", y % 10).as_str();

            for x in 0..self.width {
                let piece = &self.grid[&Coord::new(x as i32, y as i32)];
                buf += piece.to_string().as_str();
            }
            buf += "\n";
        }

        f.write_str(buf.as_str())
    }
}

// Iterate over each unit in reading order. On each's turn:
// 1. Identify all possible targets. If none, game is over.
// 2. Idenitfy all free/open squares in range of each target.
// 4. If the unit is not already in range, it moves one step towards the closest free square.
// 5. If the unit is currently in range it attacks.
