use anyhow::{bail, Result};
use core::panic;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Write};
use std::io::{self, Read};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut game: Game = input.parse()?;
    let shortest_paths = game.shortest_paths(Point::new(4, 1));
    println!("Calculated {} shortest paths", shortest_paths.len());

    println!("{}", game);
    for point in shortest_paths.keys() {
        game.grid.insert(*point, Object::Marker('?'));
    }
    println!("{}", game);

    // part1(input)?;

    Ok(())
}

fn part1(input: String) -> Result<()> {
    let mut game: Game = input.parse()?;
    loop {
        match game.play_round() {
            GameOutcome::InProgress(_) => (),
            GameOutcome::Complete(score) => {
                println!("Game complete: {:?}", score);
                break;
            },
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Combatant {
    kind: CombatantKind,
    health: usize,
    attack_power: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CombatantKind {
    Elf,
    Goblin,
}

impl Combatant {
    const DEFAULT_HEALTH: usize = 200;
    const DEFAULT_ATTACK_POWER: usize = 3;

    fn new(kind: CombatantKind) -> Combatant {
        Combatant {
            kind,
            health: Combatant::DEFAULT_HEALTH,
            attack_power: Combatant::DEFAULT_ATTACK_POWER,
        }
    }

    fn new_elf() -> Combatant {
        Combatant::new(CombatantKind::Elf)
    }

    fn new_goblin() -> Combatant {
        Combatant::new(CombatantKind::Goblin)
    }

    fn is_enemy(&self, other: &Combatant) -> bool {
        self.kind != other.kind
    }
}

#[derive(PartialEq, Eq)]
enum Object {
    Combatant(Combatant),
    Wall,
    Empty,
    Marker(char),
}

impl Object {
    fn is_empty(&self) -> bool {
        matches!(*self, Object::Empty | Object::Marker(_))
    }

    fn is_enemy(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Combatant(c1), Object::Combatant(c2)) => c1.is_enemy(c2),
            _ => false,
        }
    }

    fn is_combatant(&self) -> bool {
        matches!(self, Object::Combatant { .. })
    }

    fn unwrap(&self) -> &Combatant {
        match self {
            Object::Combatant(c) => c,
            _ => panic!("expected combatant but got: {}", self),
        }
    }
}

impl FromStr for Object {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "E" => Ok(Object::Combatant(Combatant::new_elf())),
            "G" => Ok(Object::Combatant(Combatant::new_goblin())),
            "#" => Ok(Object::Wall),
            "." => Ok(Object::Empty),
            _ => bail!("unknown object: {}", s),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Object::Combatant(c) if c.kind == CombatantKind::Elf => 'E',
            Object::Combatant(_) => 'G',
            Object::Wall => '#',
            Object::Empty => '.',
            Object::Marker(ch) => *ch,
        };

        f.write_char(ch)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    // Returns points in range of this point in reading order.
    fn in_range(&self) -> Vec<Point> {
        vec![
            Point::new(self.x, self.y - 1), // Above.
            Point::new(self.x - 1, self.y), // Left.
            Point::new(self.x + 1, self.y), // Right.
            Point::new(self.x, self.y + 1), // Below.
        ]
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.y.cmp(&other.y) {
            std::cmp::Ordering::Equal => self.x.cmp(&other.x),
            v => v,
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct Game {
    grid: HashMap<Point, Object>,
    width: usize,
    height: usize,
    rounds: usize,
}

enum GameOutcome {
    InProgress(Score),
    Complete(Score),
}

#[derive(Debug)]
struct Score {
    result: HashMap<CombatantKind, usize>,
    rounds: usize,
}

impl Game {
    // Returns enemy points in range of the specified point, in the order
    // that they should be attacked.
    fn enemies_in_range(&self, point: Point) -> Vec<Point> {
        let obj = &self.grid[&point];
        let mut points = point
            .in_range()
            .into_iter()
            .filter(|p| obj.is_enemy(&self.grid[p]))
            .collect::<Vec<_>>();

        points.sort_unstable_by(|p1, p2| {
            let c1 = self.grid[p1].unwrap();
            let c2 = self.grid[p2].unwrap();
            match c1.health.cmp(&c2.health) {
                std::cmp::Ordering::Equal => p1.cmp(p2),
                v => v,
            }
        });

        points
    }

    // Returns empty points in range of the specified point in reading order.
    fn empty_in_range(&self, point: Point) -> Vec<Point> {
        point
            .in_range()
            .into_iter()
            .filter(|p| matches!(self.grid.get(p), Some(obj) if obj.is_empty()))
            .collect::<Vec<_>>()
    }

    // Returns all empty points on the grid in reading order.
    fn empty_points(&self) -> Vec<Point> {
        self.filter_points(|obj| obj.is_empty())
    }

    // Returns all combatant points in reading order.
    fn combatant_points(&self) -> Vec<Point> {
        self.filter_points(|obj| obj.is_combatant())
    }

    // Returns all points of enemies of the specified combatant kind in reading order.
    fn enemy_points(&self, enemy_of: CombatantKind) -> Vec<Point> {
        self.filter_points(|obj| {
            if let Object::Combatant(c) = obj {
                c.kind != enemy_of
            } else {
                false
            }
        })
    }

    // Returns all points on the grid, in reading order, filtered by the predicate.
    fn filter_points<P>(&self, predicate: P) -> Vec<Point>
    where
        P: Fn(&Object) -> bool,
    {
        let mut points = vec![];
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::new(x as i32, y as i32);
                if predicate(&self.grid[&point]) {
                    points.push(point);
                }
            }
        }

        points
    }

    fn shortest_paths(&self, from: Point) -> HashMap<Point, Vec<Point>> {
        let mut shortest_paths = HashMap::new();
        shortest_paths.insert(from, vec![]);

        let mut unvisited = self.empty_points().into_iter().collect::<HashSet<_>>();
        let mut current = from;

        loop {
            println!("shortest_path: {:?}", current);

            let empty_neighbors = self
                .empty_in_range(current)
                .into_iter()
                .collect::<HashSet<_>>();

            let unvisited_neighbors = empty_neighbors
                .intersection(&unvisited)
                .cloned()
                .collect::<HashSet<_>>();

            let current_path = shortest_paths[&current].clone();
            let current_cost = current_path.len();
            let neighbor_cost = current_cost + 1;

            let mut possible_paths = vec![];

            for neighbor in &unvisited_neighbors {
                let existing_neighbor_cost = shortest_paths
                    .get(neighbor)
                    .map(|v| v.len())
                    .unwrap_or(usize::MAX);

                let mut neighbor_path = shortest_paths.get(neighbor).cloned().unwrap_or_default();

                if neighbor_cost < existing_neighbor_cost {
                    neighbor_path = current_path.clone();
                    neighbor_path.push(*neighbor);
                    shortest_paths.insert(*neighbor, neighbor_path.clone());
                }

                possible_paths.push(neighbor_path);
            }

            possible_paths.sort_unstable_by(|p1, p2| match p1.len().cmp(&p2.len()) {
                std::cmp::Ordering::Equal => p1.last().unwrap().cmp(p2.last().unwrap()),
                v => v,
            });

            if let Some(next) = possible_paths.first() {
                unvisited.remove(&current);
                current = *next.last().unwrap();
            } else {
                // We are finished. At this point, we have calculated a shortest path to every
                // empty point on the grid. The "unvisited" set may not (and probably will not)
                // be empty, however, as we may never have decided to move to those remaining
                // points (there were cheaper alternatives).
                break;
            }
        }

        shortest_paths
    }

    fn try_attack(&mut self, point: Point) -> bool {
        let subject = self.grid[&point].unwrap();
        if let Some(enemy_point) = self.enemies_in_range(point).first() {
            println!("Attacking {:?}", enemy_point);

            let mut enemy = *self.grid[enemy_point].unwrap();
            enemy.health = enemy.health.saturating_sub(subject.attack_power);
            if enemy.health == 0 {
                self.grid.insert(*enemy_point, Object::Empty);
            } else {
                self.grid.insert(*enemy_point, Object::Combatant(enemy));
            }

            true
        } else {
            false
        }
    }

    fn outcome(&self) -> GameOutcome {
        // Calculate the number of remaining health points for each kind of combatant.
        let mut points = HashMap::new();
        for subject_point in self.combatant_points() {
            let subject = *self.grid[&subject_point].unwrap();
            let points = points.entry(subject.kind).or_insert(0);
            *points += subject.health;
        }

        // If only one side has a score greater than zero (i.e., they're still alive) then
        // the game is complete and we have a winner.
        let game_complete = points.values().filter(|&v| *v > 0).count() == 1;

        let score = Score { result: points, rounds: self.rounds };
        if game_complete {
            GameOutcome::Complete(score)
        } else {
            GameOutcome::InProgress(score)
        }
    }

    fn play_round(&mut self) -> GameOutcome {
        println!("{}", self);

        // Iterate over each combatant in reading order.
        for subject_point in self.combatant_points() {
            println!("Subject point is {:?}", subject_point);

            // Get the subject combatant at the current point.
            let subject = *self.grid[&subject_point].unwrap();

            // Get the list of enemy points. If there are none then the game has been won by
            // the side of the current subject so we return immediately.
            let enemy_points = self.enemy_points(subject.kind);
            if enemy_points.is_empty() {
                return self.outcome();
            }

            // If the combatant is in a position to attack they do so and don't move.
            if self.try_attack(subject_point) {
                continue;
            }

            // Possible destinations for the subject to move towards are all empty points in range
            // of an enemy combatant as these are the points from which an attack can be launched.
            let mut possible_destinations = vec![];
            for enemy_point in enemy_points {
                let mut attack_points = self
                    .empty_in_range(enemy_point)
                    .into_iter()
                    .collect::<Vec<_>>();
                possible_destinations.append(&mut attack_points);
            }

            println!("Found {} possible destinations for subject", possible_destinations.len());

            // Calculate the shortest paths from the subject's position to every empty point.
            let shortest_paths = self.shortest_paths(subject_point);

            println!("Calculated {} shortest paths", shortest_paths.len());

            // Choose the path to one of the previously calculated possible destinations (i.e.,
            // the attack positions) that will require the smallest number of moves.
            let mut chosen_path: Option<&Vec<Point>> = None;
            for destination in possible_destinations {
                let path = &shortest_paths[&destination];
                if path.len() < chosen_path.map(|v| v.len()).unwrap_or(usize::MAX) {
                    chosen_path = Some(path);
                }
            }

            // If a path was found, take the first step.
            if let Some(chosen_path) = chosen_path {
                println!("Moving {:?} to {:?}", subject_point, chosen_path[0]);
                self.grid.insert(chosen_path[0], Object::Combatant(subject));
                self.grid.insert(subject_point, Object::Empty);

                // If the path was only a single step in length then we must have arrived at
                // the attack position so we launch an attack.
                if chosen_path.len() == 1 {
                    self.try_attack(subject_point);
                }
            } else {
                println!("Didn't mmove anywhere");
            }

            println!("{}", self);
            println!();
        }

        self.rounds += 1;
        self.outcome()
    }
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = HashMap::new();
        let mut x_max = 0;
        for (y, line) in s.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let obj = ch.to_string().parse()?;
                grid.insert(Point::new(x as i32, y as i32), obj);
                x_max = x_max.max(x);
            }
        }

        Ok(Game {
            grid,
            width: x_max + 1,
            height: s.lines().count(),
            rounds: 0,
        })
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
                let obj = &self.grid[&Point::new(x as i32, y as i32)];
                buf += obj.to_string().as_str();
            }
            buf += "\n";
        }

        f.write_str(buf.as_str())
    }
}
