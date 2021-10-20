use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, VecDeque},
    io::{self, stdout, Read, Write},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let games = input
        .lines()
        .map(|v| v.parse())
        .collect::<Result<Vec<Game>>>()?;

    for (num, game) in games.iter().enumerate() {
        let score = high_score(game)?;
        println!("Game {}: High score is {}", num + 1, score);
    }

    Ok(())
}

fn high_score(game: &Game) -> Result<u32> {
    let mut circle: VecDeque<u32> = VecDeque::new();
    let mut current_marble_index: usize = 0;
    let mut current_player: u32 = 1;
    let mut points_by_player: HashMap<u32, u32> = HashMap::new();
    let mut percent_complete: u8 = 0;
    circle.push_back(0);

    for turn in 1..game.total_marbles {
        if turn % 23 == 0 {
            let i = current_marble_index as i32 - 7;
            let n = circle.len() as i32;
            current_marble_index = (((i % n) + n) % n) as usize;
            let points = turn + circle.remove(current_marble_index).unwrap();
            *points_by_player.entry(current_player).or_default() += points;
        } else {
            current_marble_index = (current_marble_index + 2) % circle.len();
            circle.insert(current_marble_index, turn);
        }

        current_player = (current_player % game.total_players) + 1;

        let new_percent_complete = (turn * 100 / (game.total_marbles - 1)) as u8;
        if new_percent_complete > percent_complete {
            percent_complete = new_percent_complete;
            print!("\rCompleted {}%", percent_complete);
            stdout().flush().unwrap();
        }
    }

    println!();

    Ok(*points_by_player.values().max().unwrap_or(&0))
}

struct Game {
    total_players: u32,
    total_marbles: u32,
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

        Ok(Game {
            total_players: caps["players"].parse()?,
            total_marbles: caps["points"].parse::<u32>()? + 1,
        })
    }
}
