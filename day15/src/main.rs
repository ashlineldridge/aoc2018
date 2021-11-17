use anyhow::{bail, Result};
use core::panic;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Write};
use std::io::{self, Read};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    part1(input.clone())?;
    part2(input)?;

    Ok(())
}

fn part1(input: String) -> Result<()> {
    let mut game: Game = input.parse()?;
    loop {
        if let GameState::Complete(score) = game.play_round() {
            println!("Game complete: {:?}", score);
            println!("Game outcome: {}", score.outcome());
            break;
        }
    }

    Ok(())
}

fn part2(input: String) -> Result<()> {
    let game: Game = input.parse()?;
    let initial_elf_count = game.outcome().score().remaining_combatants[&CombatantKind::Elf];
    let mut attack_power = Combatant::DEFAULT_ATTACK_POWER + 1;

    loop {
        let mut game = game.clone();

        let attack_powers = HashMap::from([
            (CombatantKind::Elf, attack_power),
            (CombatantKind::Goblin, Combatant::DEFAULT_ATTACK_POWER),
        ]);

        game.update_attack_power(attack_powers);

        let score = loop {
            if let GameState::Complete(score) = game.play_round() {
                break score;
            }
        };

        println!("Elf attack power {}: {:?}", attack_power, score);

        let final_elf_count = score.remaining_combatants[&CombatantKind::Elf];
        if final_elf_count == initial_elf_count {
            // The elves finally won without losing any combatants!
            println!("Final game outcome: {}", score.outcome());
            break;
        }

        attack_power += 1;
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

impl CombatantKind {
    fn enemy(&self) -> CombatantKind {
        match self {
            CombatantKind::Elf => CombatantKind::Goblin,
            CombatantKind::Goblin => CombatantKind::Elf,
        }
    }
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
        other.kind == self.kind.enemy()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Object {
    Combatant(Combatant),
    Wall,
    Empty,
}

impl Object {
    fn is_empty(&self) -> bool {
        matches!(*self, Object::Empty)
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

    fn as_combatant(&self) -> &Combatant {
        match self {
            Object::Combatant(c) => c,
            _ => panic!("expected combatant but got: {}", self),
        }
    }

    fn as_combatant_mut(&mut self) -> &mut Combatant {
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

#[derive(Clone)]
struct Game {
    grid: HashMap<Point, Object>,
    width: usize,
    height: usize,
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
            let c1 = self.grid[p1].as_combatant();
            let c2 = self.grid[p2].as_combatant();
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

    // Uses Dijkstra's algorithm to calculate the shortest path from the specified point to every
    // free point on the grid. See: https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm.
    fn shortest_paths(&mut self, from: Point) -> HashMap<Point, Vec<Point>> {
        let mut shortest_paths = HashMap::new();

        // The shortest path from the starting point to itself is an empty path.
        shortest_paths.insert(from, vec![]);

        // Keep track of all the empty points on the grid which we have not yet visited. Visiting
        // a point means that we have calculated the shortest path to that point and we will not
        // consider it again.
        let mut unvisited = self.empty_points().into_iter().collect::<HashSet<_>>();

        // Set the current point to the starting point and loop until we have calculated the
        // shortest path to every free point which can be reached (some may be blocked).
        let mut current = from;
        loop {
            // The neighbors of the current point which are empty.
            let empty_neighbors = self
                .empty_in_range(current)
                .into_iter()
                .collect::<HashSet<_>>();

            // The neighbors of the current point which are empty and unvisited.
            let unvisited_neighbors = empty_neighbors
                .intersection(&unvisited)
                .cloned()
                .collect::<HashSet<_>>();

            // The path that we took to reach the current point.
            let current_path = shortest_paths[&current].clone();

            // The distance from the starting position to the current point.
            let current_distance = current_path.len();

            // The distance from the starting position to each neighbor of the current point.
            let neighbor_distance = current_distance + 1;

            // For each unvisited neighbor of the current point check whether the distance of the
            // path to the neighbor that runs through the current point is less than any previously
            // calculated tentative distance (i.e., the length of the path that we previously
            // calculated for the neighbor when we last encountered it (or "infinity" / usize::MAX
            // if we have not encountered the neighbor before)). If the new distance is less than
            // the old one, record the new path as the tentative shortest path for the neighbor.
            for neighbor in &unvisited_neighbors {
                let existing_neighbor_cost = shortest_paths
                    .get(neighbor)
                    .map(|v| v.len())
                    .unwrap_or(usize::MAX);

                if neighbor_distance < existing_neighbor_cost {
                    let mut neighbor_path = current_path.clone();
                    neighbor_path.push(*neighbor);
                    shortest_paths.insert(*neighbor, neighbor_path);
                }
            }

            // Consider the current point to be "visited". The shortest path recorded for this
            // point is now final.
            unvisited.remove(&current);

            // Dijkstra's algorithm says to set the current point to the point that is closest
            // to the starting position that has not yet been visited.

            // Collect all recorded destinations that have a recorded path (this will include
            // both visited points and unvisited neighbors of visited points).
            let all_destinations = shortest_paths.keys().cloned().collect::<HashSet<_>>();

            // Collect all of the recorded destinations that have not yet been visited.
            let unvisited_destinations = all_destinations
                .intersection(&unvisited)
                .cloned()
                .collect::<HashSet<_>>();

            // Calculate the distance to the closest unvisited destination.
            let closest_distance = unvisited_destinations
                .iter()
                .map(|v| shortest_paths[v].len())
                .min();

            // A number of unvisited destinations could have the same minimum distance so we sort
            // them into reading order and select the first one as the next current point.
            if let Some(closest_distance) = closest_distance {
                let mut closest_destinations = unvisited_destinations
                    .into_iter()
                    .filter(|p| shortest_paths[p].len() == closest_distance)
                    .collect::<Vec<_>>();
                closest_destinations.sort_unstable();
                current = *closest_destinations.first().unwrap();
            } else {
                // There are no more points on the grid that it is possible to move to. We're done.
                break;
            }
        }

        shortest_paths
    }

    fn try_attack(&mut self, point: Point) -> AttackResult {
        let subject = self.grid[&point].as_combatant();
        if let Some(enemy_point) = self.enemies_in_range(point).first() {
            let mut enemy = *self.grid[enemy_point].as_combatant();
            enemy.health = enemy.health.saturating_sub(subject.attack_power);

            if enemy.health == 0 {
                self.grid.insert(*enemy_point, Object::Empty);
                AttackResult::Killed(*enemy_point)
            } else {
                self.grid.insert(*enemy_point, Object::Combatant(enemy));
                AttackResult::Hit(*enemy_point)
            }
        } else {
            AttackResult::Missed
        }
    }

    fn outcome(&self) -> GameState {
        // Calculate the number of remaining health points for each kind of combatant.
        let mut remaining_health =
            HashMap::from([(CombatantKind::Elf, 0), (CombatantKind::Goblin, 0)]);

        // We'll also calculate the number of remaining combatants on each side.
        let mut remaining_combatants = remaining_health.clone();

        for subject_point in self.combatant_points() {
            let subject = *self.grid[&subject_point].as_combatant();
            let points = remaining_health.entry(subject.kind).or_insert(0);
            *points += subject.health;

            let remaining = remaining_combatants.entry(subject.kind).or_insert(0);
            *remaining += 1;
        }

        // If one side has zero remaining health then they are the loser and the the other
        // side must be the winner.
        let winner = remaining_health.iter().find_map(|(&kind, &health)| {
            if health == 0 {
                Some(kind.enemy())
            } else {
                None
            }
        });

        let score = Score {
            remaining_combatants,
            total_health: remaining_health,
            rounds_played: self.rounds,
            winner,
        };

        // If we have a winner the game is over.
        if winner.is_some() {
            GameState::Complete(score)
        } else {
            GameState::InProgress(score)
        }
    }

    fn play_round(&mut self) -> GameState {
        // For each round, keep track of the points on the grid where a combatant was killed.
        let mut kill_points = HashSet::new();

        // Iterate over each combatant in reading order.
        for subject_point in self.combatant_points() {
            if kill_points.contains(&subject_point) {
                // At the beginning of the round there was a combatant at the current subject
                // point but they have been killed and removed from the board so we continue.
                continue;
            }

            // Get the subject combatant at the current point.
            let subject = *self.grid[&subject_point].as_combatant();

            // Get the list of enemy points. If there are none then the game has been won by
            // the side of the current subject so we return immediately.
            let enemy_points = self.enemy_points(subject.kind);
            if enemy_points.is_empty() {
                return self.outcome();
            }

            // If the combatant is in a position to attack they do so and don't move.
            match self.try_attack(subject_point) {
                AttackResult::Missed => (),
                AttackResult::Hit(_) => continue,
                AttackResult::Killed(p) => {
                    kill_points.insert(p);
                    continue;
                }
            };

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

            // Calculate the shortest paths from the subject's position to every empty point.
            let shortest_paths = self.shortest_paths(subject_point);

            // Choose the path to one of the previously calculated possible destinations (i.e.,
            // the attack positions) that will require the smallest number of moves.
            let mut chosen_path: Option<&Vec<Point>> = None;
            for destination in possible_destinations {
                if let Some(path) = shortest_paths.get(&destination) {
                    if path.len() < chosen_path.map(|v| v.len()).unwrap_or(usize::MAX) {
                        chosen_path = Some(path);
                    }
                }
            }

            // If a path was found, take the first step.
            if let Some(chosen_path) = chosen_path {
                self.grid.insert(subject_point, Object::Empty);

                let subject_point = chosen_path[0];
                self.grid.insert(subject_point, Object::Combatant(subject));

                // If the path was only a single step in length then we must have arrived at
                // the attack position so we launch an attack.
                if chosen_path.len() == 1 {
                    if let AttackResult::Killed(p) = self.try_attack(subject_point) {
                        kill_points.insert(p);
                    }
                }
            }
        }

        self.rounds += 1;
        self.outcome()
    }

    fn update_attack_power(&mut self, attack_powers: HashMap<CombatantKind, usize>) {
        for point in self.combatant_points() {
            let combatant = self.grid.get_mut(&point).unwrap().as_combatant_mut();
            combatant.attack_power = *attack_powers
                .get(&combatant.kind)
                .unwrap_or(&Combatant::DEFAULT_ATTACK_POWER);
        }
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

#[derive(Clone, Debug)]
enum GameState {
    InProgress(Score),
    Complete(Score),
}

impl GameState {
    fn score(&self) -> &Score {
        match self {
            GameState::InProgress(score) => score,
            GameState::Complete(score) => score,
        }
    }
}

#[derive(Clone, Debug)]
struct Score {
    remaining_combatants: HashMap<CombatantKind, usize>,
    total_health: HashMap<CombatantKind, usize>,
    rounds_played: usize,
    winner: Option<CombatantKind>,
}

impl Score {
    fn outcome(&self) -> usize {
        if let Some(winner) = self.winner {
            self.total_health[&winner] * self.rounds_played
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AttackResult {
    Missed,
    Hit(Point),
    Killed(Point),
}
