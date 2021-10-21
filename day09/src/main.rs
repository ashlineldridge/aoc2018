use anyhow::{anyhow, bail, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut games = input
        .lines()
        .map(|v| v.parse())
        .collect::<Result<Vec<Game>>>()?;

    for (num, game) in games.iter_mut().enumerate() {
        let (winner_id, score) = game.run()?;
        println!(
            "Game {}: Player {} won with high score is {}",
            num + 1,
            winner_id,
            score
        );
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Marble {
    value: MarbleValue,
    next_index: usize,
    prev_index: usize,
}

type MarbleValue = u32;

type Player = u32;

type Score = HashMap<Player, u32>;

struct Game {
    total_players: usize,
    total_marbles: usize,
    circle: Vec<Marble>,
    current_marble_index: usize,
    next_marble_value: MarbleValue,
    next_player: Player,
    score: Score,
}

impl Game {
    fn new(total_players: usize, total_marbles: usize) -> Game {
        Game {
            total_players,
            total_marbles,
            circle: vec![Marble {
                value: 0,
                next_index: 0,
                prev_index: 0,
            }],
            current_marble_index: 0,
            next_marble_value: 1,
            next_player: 1,
            score: Score::new(),
        }
    }

    fn run(&mut self) -> Result<(Player, u32)> {
        if self.total_marbles == 0 {
            bail!("Game can't be played without marbles");
        }
        if self.total_players == 0 {
            bail!("Game can't be played without players");
        }

        while self.play_next() {}

        self.score
            .iter()
            .max_by_key(|(_, v)| **v)
            .map(|(k, v)| (*k, *v))
            .context("Game did not produce a winner")
    }

    fn play_next(&mut self) -> bool {
        if self.next_marble_value == self.total_marbles as u32 {
            return false;
        }

        if self.next_marble_value % 23 == 0 {
            let prev_7 = self.prev(7).clone();
            self.prev(8).next_index = prev_7.next_index;
            self.prev(6).prev_index = prev_7.prev_index;
            self.current_marble_index = prev_7.next_index;

            let points = (self.next_marble_value + prev_7.value) as u32;
            *self.score.entry(self.next_player).or_default() += points;
        } else {
            let next_marble = Marble {
                value: self.next_marble_value,
                next_index: self.next(1).next_index,
                prev_index: self.curr().next_index,
            };
            let next_marble_index = self.circle.len();

            self.next(2).prev_index = next_marble_index;
            self.next(1).next_index = next_marble_index;

            self.circle.push(next_marble);
            self.current_marble_index = next_marble_index;
        }

        self.next_marble_value += 1;
        self.next_player = (self.next_player % self.total_players as u32) + 1;

        true
    }

    fn curr(&mut self) -> &mut Marble {
        self.circle.get_mut(self.current_marble_index).unwrap()
    }

    fn next(&mut self, n: usize) -> &mut Marble {
        let mut next_index = self.current_marble_index;
        for _ in 0..n {
            next_index = self.circle[next_index].next_index;
        }
        self.circle.get_mut(next_index).unwrap()
    }

    fn prev(&mut self, n: usize) -> &mut Marble {
        let mut prev_index = self.current_marble_index;
        for _ in 0..n {
            prev_index = self.circle[prev_index].prev_index;
        }
        self.circle.get_mut(prev_index).unwrap()
    }
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(?P<players>\d+) players; last marble is worth (?P<points>\d+) points$"
            )
            .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid line: {}", s))?;

        Ok(Game::new(caps["players"].parse()?, caps["points"].parse::<usize>()? + 1))
    }
}
