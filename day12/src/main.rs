use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fmt::Display,
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut plant_state: PlantState = input.parse()?;
    for _ in 0..20 {
        println!("{}", plant_state);
        plant_state.advance();
    }

    Ok(())
}

#[derive(Debug)]
struct PlantState {
    state: Vec<bool>,
    rules: Vec<PlantRule>,
}

impl PlantState {
    fn advance(&mut self) {
        let c = self.state.clone();
        for (i, v) in self.state.iter_mut().enumerate() {
            for r in &self.rules {
                if let Some(outcome) = r.eval(&c, i as i32) {
                    *v = outcome;
                    break;
                }
            }
        }
    }
}

impl FromStr for PlantState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<&str>>();
        let state = lines
            .get(0)
            .and_then(|line| line.split_whitespace().last())
            .map(|pattern| pattern.chars().map(|c| c == '#').collect::<Vec<bool>>())
            .context("Invalid input")?;
        let rules: Result<Vec<PlantRule>> = lines[1..]
            .iter()
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(line.parse())
                }
            })
            .collect();
        let rules = rules?;

        Ok(PlantState { state, rules })
    }
}

impl Display for PlantState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state: String = self
            .state
            .iter()
            .map(|v| if *v { '#' } else { '.' })
            .collect();
        f.write_str(&state)
    }
}

#[derive(Debug)]
struct PlantRule {
    pattern: Vec<bool>,
    outcome: bool,
}

impl PlantRule {
    fn matches(&self, state: &[bool], index: i32) -> bool {
        let siblings = self.pattern.len() as i32 / 2;
        for i in index - siblings..=index + siblings {
            let mut s = false;
            if i >= 0 && i < state.len() as i32 {
                s = state[i as usize];
            }

            if self.pattern[(i - index + siblings) as usize] != s {
                return false;
            }
        }

        true
    }

    fn eval(&self, state: &[bool], index: i32) -> Option<bool> {
        if self.matches(state, index) {
            Some(self.outcome)
        } else {
            None
        }
    }
}

impl FromStr for PlantRule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(?P<pattern>[#\.]{5}) => (?P<outcome>#|.)$").unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid line: {}", s))?;

        Ok(PlantRule {
            pattern: caps["pattern"].chars().map(|c| c == '#').collect(),
            outcome: &caps["outcome"] == "#",
        })
    }
}
