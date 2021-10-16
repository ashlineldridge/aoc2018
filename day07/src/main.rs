use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let deps = read_deps(&input)?;
    let answer = part1(&deps)?;
    println!("Answer to part 1: {}", answer);

    let answer = part2(&deps)?;
    println!("Answer to part 2: {}", answer);

    Ok(())
}

fn part1(deps: &DepsByStep) -> Result<String> {
    let mut result = String::from("");
    let mut completed_steps = HashSet::new();
    let mut available_steps = deps
        .iter()
        .filter_map(|(s, v)| if v.is_empty() { Some(*s) } else { None })
        .collect::<HashSet<Step>>();

    while !available_steps.is_empty() {
        let chosen = *available_steps.iter().max_by(|s1, s2| s2.cmp(s1)).unwrap();
        available_steps.remove(&chosen);
        completed_steps.insert(chosen);
        result += chosen.to_string().as_str();

        let unlocked_steps = deps
            .iter()
            .filter_map(|(s, v)| {
                if !completed_steps.contains(s) && v.is_subset(&completed_steps) {
                    Some(*s)
                } else {
                    None
                }
            })
            .collect::<HashSet<Step>>();
        available_steps.extend(unlocked_steps);
    }

    Ok(result)
}

fn part2(deps: &DepsByStep) -> Result<String> {

    Ok(String::from(""))
}

type Step = char;

type DepsByStep = HashMap<Step, HashSet<Step>>;

#[derive(Eq, Hash, PartialEq)]
struct DepPair(Step, Step);

impl FromStr for DepPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^Step (?P<s1>[A-Z]) must be finished before step (?P<s2>[A-Z])")
                    .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid line: {}", s))?;

        Ok(DepPair(caps["s2"].parse()?, caps["s1"].parse()?))
    }
}

fn read_deps(input: &str) -> Result<DepsByStep> {
    let pairs: Result<HashSet<DepPair>> = input.lines().map(|v| v.parse()).collect();
    let pairs = pairs?;

    let mut deps = DepsByStep::new();
    for pair in pairs {
        deps.entry(pair.0).or_default().insert(pair.1);
        deps.entry(pair.1).or_default();
    }

    Ok(deps)
}
