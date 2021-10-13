use std::io::{self, Read};

use anyhow::Result;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim();

    let polymer = part1(input);
    println!("Part 1 answer: {}", polymer.len());

    let polymer = part2(input);
    println!("Part 2 answer: {}", polymer.len());

    Ok(())
}

fn part1(polymer: &str) -> String {
    react(polymer)
}

fn part2(polymer: &str) -> String {
    let mut min: Option<String> = None;
    for v in 'a'..='z' {
        let polymer: String = polymer
            .chars()
            .filter(|c| *c != v && *c != v.to_ascii_uppercase())
            .collect();
        let polymer = react(polymer.as_str());
        println!(
            "Test({}) decomposed polymer has length: {}",
            v,
            polymer.len()
        );

        min = match min {
            Some(p) if p.len() < polymer.len() => Some(p),
            _ => Some(polymer),
        }
    }

    min.unwrap()
}

fn react(polymer: &str) -> String {
    let mut polymer: Vec<char> = polymer.chars().collect();
    let mut reacted = vec![];

    loop {
        let mut shrunk = false;
        let mut i = 1;
        while i < polymer.len() {
            let p = polymer[i - 1];
            let c = polymer[i];
            if p != c && p.to_ascii_uppercase() == c.to_ascii_uppercase() {
                shrunk = true;
                i += 2;
            } else {
                reacted.push(p);
                i += 1;
            }
        }

        if i == polymer.len() {
            reacted.push(polymer[i - 1]);
        }

        if !shrunk {
            break;
        }

        std::mem::swap(&mut polymer, &mut reacted);
        reacted.clear();
    }

    reacted.iter().collect()
}
