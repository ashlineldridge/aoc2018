use anyhow::{bail, Context, Result};
use std::{
    collections::VecDeque,
    io::{self, Read},
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let values: Result<VecDeque<u32>, _> = input
        .trim()
        .split(' ')
        .map(|v| v.parse::<u32>())
        .collect();
    let mut values = values?;

    let answer = part1(&mut values)?;
    println!("Answer to part 1 is {}", answer);

    Ok(())
}

fn part1(values: &mut VecDeque<u32>) -> Result<u32> {
    let total_children = values
        .pop_front()
        .context("expected header specifying number of child nodes")?;
    let total_entries = values
        .pop_front()
        .context("expected header specifying number of metadata entries")?
        as usize;

    let mut result = 0;
    for _ in 0..total_children {
        result += part1(values)?;
    }

    let entries = values.drain(0..total_entries).collect::<Vec<u32>>();
    if entries.len() != total_entries {
        bail!(
            "expected {} metadata entries but got {}",
            total_entries,
            entries.len()
        );
    }

    Ok(result + entries.iter().sum::<u32>())
}
