use anyhow::{bail, Result};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{Display, Write},
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut grid: Grid = input.parse()?;

    let first_crash_location = part1(&mut grid.clone());
    println!("Part 1 answer: {:?}", first_crash_location);

    let last_cart_location = part2(&mut grid);
    println!("Part 2 answer: {:?}", last_cart_location);

    Ok(())
}

fn part1(grid: &mut Grid) -> Location {
    loop {
        if let Some(cart) = grid.tick().first() {
            return cart.location;
        }
    }
}

fn part2(grid: &mut Grid) -> Location {
    loop {
        grid.tick();
        if grid.carts.len() == 1 {
            let carts = grid.carts.values().collect::<Vec<&Cart>>();
            return carts.first().unwrap().location;
        }
    }
}

const CART_INTERSECTION_CHOICES: [IntersectionChoice; 3] = [
    IntersectionChoice::Left,
    IntersectionChoice::Straight,
    IntersectionChoice::Right,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IntersectionChoice {
    Left,
    Right,
    Straight,
}

#[derive(Clone, Debug)]
struct Cart {
    location: Location,
    facing: Facing,
    choice_index: usize,
}

impl Cart {
    fn new(location: Location, facing: Facing) -> Cart {
        Cart {
            location,
            facing,
            choice_index: 0,
        }
    }

    fn move_next(&mut self, track: &TrackMap) {
        if track[&self.location] == TrackPiece::Intersection {
            let next_facing = match CART_INTERSECTION_CHOICES[self.choice_index] {
                IntersectionChoice::Left => self.facing.rotate_left(),
                IntersectionChoice::Right => self.facing.rotate_right(),
                IntersectionChoice::Straight => self.facing,
            };
            self.facing = next_facing;
            self.choice_index = (self.choice_index + 1) % CART_INTERSECTION_CHOICES.len();
        }

        let next_location = match self.facing {
            Facing::Up => self.location.up(),
            Facing::Down => self.location.down(),
            Facing::Left => self.location.left(),
            Facing::Right => self.location.right(),
        };

        self.location = next_location;

        let next_facing = match track[&self.location] {
            TrackPiece::TopLeft => match self.facing {
                Facing::Up => Facing::Right,
                Facing::Left => Facing::Down,
                _ => panic!("Invalid state"),
            },
            TrackPiece::TopRight => match self.facing {
                Facing::Up => Facing::Left,
                Facing::Right => Facing::Down,
                _ => panic!("Invalid state"),
            },
            TrackPiece::BottomLeft => match self.facing {
                Facing::Down => Facing::Right,
                Facing::Left => Facing::Up,
                _ => panic!("Invalid state"),
            },
            TrackPiece::BottomRight => match self.facing {
                Facing::Down => Facing::Left,
                Facing::Right => Facing::Up,
                _ => panic!("Invalid state"),
            },
            _ => self.facing,
        };

        self.facing = next_facing;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Location {
    x: i32,
    y: i32,
}

impl Location {
    fn new(x: i32, y: i32) -> Location {
        Location { x, y }
    }

    fn left(&self) -> Location {
        Location {
            x: self.x - 1,
            y: self.y,
        }
    }

    fn right(&self) -> Location {
        Location {
            x: self.x + 1,
            y: self.y,
        }
    }

    fn up(&self) -> Location {
        Location {
            x: self.x,
            y: self.y - 1,
        }
    }

    fn down(&self) -> Location {
        Location {
            x: self.x,
            y: self.y + 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Facing {
    Up,
    Down,
    Left,
    Right,
}

impl Facing {
    fn is_vertical(&self) -> bool {
        *self == Facing::Up || *self == Facing::Down
    }

    fn rotate_left(&self) -> Facing {
        match self {
            Facing::Up => Facing::Left,
            Facing::Down => Facing::Right,
            Facing::Left => Facing::Down,
            Facing::Right => Facing::Up,
        }
    }

    fn rotate_right(&self) -> Facing {
        match self {
            Facing::Up => Facing::Right,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
            Facing::Right => Facing::Down,
        }
    }
}

impl FromStr for Facing {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "^" => Ok(Facing::Up),
            "v" => Ok(Facing::Down),
            "<" => Ok(Facing::Left),
            ">" => Ok(Facing::Right),
            _ => bail!("Invalid facing direction: {}", s),
        }
    }
}

impl Display for Facing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Facing::Up => '^',
            Facing::Down => 'v',
            Facing::Left => '<',
            Facing::Right => '>',
        };

        f.write_char(ch)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrackPiece {
    Horizontal,
    Vertical,
    Intersection,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Display for TrackPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            TrackPiece::Horizontal => '-',
            TrackPiece::Vertical => '|',
            TrackPiece::Intersection => '+',
            TrackPiece::TopLeft | TrackPiece::BottomRight => '/',
            TrackPiece::TopRight | TrackPiece::BottomLeft => '\\',
        };

        f.write_char(ch)
    }
}

type TrackMap = HashMap<Location, TrackPiece>;
type CartMap = HashMap<Location, Cart>;

#[derive(Clone)]
struct Grid {
    track: TrackMap,
    carts: CartMap,
    width: usize,
    height: usize,
}

impl Grid {
    fn tick(&mut self) -> Vec<Cart> {
        let mut carts_ordered = self.carts.values().cloned().collect::<Vec<_>>();
        carts_ordered.sort_unstable_by(|c1, c2| {
            match c1.location.y {
                y if y < c2.location.y => Ordering::Less,
                y if y == c2.location.y => c1.location.x.cmp(&c2.location.x),
                _ => Ordering::Greater,
            }
        });

        let mut crashed_carts = vec![];
        for cart in &mut carts_ordered {
            if self.carts.get(&cart.location).is_none() {
                // The current cart was involved in a crash and has been removed from the track.
                continue;
            }

            let old_location = cart.location;
            self.carts.remove(&old_location);

            cart.move_next(&self.track);
            if let Some(other_cart) = self.carts.remove(&cart.location) {
                crashed_carts.push(cart.clone());
                crashed_carts.push(other_cart);
            } else {
                self.carts.insert(cart.location, cart.clone());
            }
        }

        crashed_carts
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut raw = HashMap::new();
        let lines = s.lines().collect::<Vec<&str>>();
        let mut max_width = 0;
        for (y, line) in lines.iter().enumerate() {
            let chars = line.chars().collect::<Vec<char>>();
            for (x, &ch) in chars.iter().enumerate() {
                if !ch.is_whitespace() {
                    raw.insert(Location::new(x as i32, y as i32), ch);
                }
            }

            max_width = max_width.max(chars.len());
        }

        let mut track = TrackMap::new();
        let mut carts = CartMap::new();
        for (&location, &ch) in &raw {
            let piece = match ch {
                '-' => TrackPiece::Horizontal,
                '|' => TrackPiece::Vertical,
                '+' => TrackPiece::Intersection,
                '/' => {
                    let ch_right = raw.get(&location.right()).unwrap_or(&' ');
                    match ch_right {
                        '-' | '+' | '<' | '>' => TrackPiece::TopLeft,
                        _ => TrackPiece::BottomRight,
                    }
                }
                '\\' => {
                    let ch_left = raw.get(&location.left()).unwrap_or(&' ');
                    match ch_left {
                        '-' | '+' | '<' | '>' => TrackPiece::TopRight,
                        _ => TrackPiece::BottomLeft,
                    }
                }
                '^' | 'v' | '<' | '>' => {
                    let facing = ch.to_string().parse()?;
                    carts.insert(location, Cart::new(location, facing));

                    // Here, we make the assumption that no cart starts on
                    // an intersection or on a corner piece of the track.
                    if facing.is_vertical() {
                        TrackPiece::Vertical
                    } else {
                        TrackPiece::Horizontal
                    }
                }
                _ => bail!("Unexpected track piece: {}", ch),
            };

            track.insert(location, piece);
        }

        Ok(Grid {
            track,
            carts,
            width: max_width,
            height: lines.len(),
        })
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let location = Location::new(x as i32, y as i32);
                if let Some(cart) = self.carts.get(&location) {
                    out += cart.facing.to_string().as_str();
                } else if let Some(piece) = self.track.get(&location) {
                    out += piece.to_string().as_str();
                } else {
                    out += " ";
                }
            }

            out += "\n";
        }

        f.write_str(out.as_str())
    }
}
