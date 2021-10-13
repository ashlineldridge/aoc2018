use std::io::{self, Read};

use anyhow::Result;

const NULL_CHAR: char = '0';

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let polymer = decompose(&input)?;
    println!("Decomposed polymer has length: {}", polymer.len());

    let polymer = decompose_max(&input)?;
    println!("Maximally decomposed polymer has length: {}", polymer.len());

    Ok(())
}

fn decompose(polymer: &str) -> Result<String> {
    let mut chars: Vec<char> = polymer.trim().chars().collect();

    loop {
        let mut shrunk = false;
        let mut prev: Option<&mut char> = None;
        for c in chars.iter_mut().filter(|c| **c != NULL_CHAR) {
            if let Some(p) = prev {
                if p != c && p.to_ascii_uppercase() == c.to_ascii_uppercase() {
                    *p = NULL_CHAR;
                    *c = NULL_CHAR;
                    shrunk = true;
                    prev = None;
                    continue;
                }
            }

            prev = Some(c);
        }

        if !shrunk {
            break;
        }
    }

    Ok(chars.iter().filter(|c| **c != NULL_CHAR).collect())
}

fn decompose_max(polymer: &str) -> Result<String> {
    let mut min: Option<String> = None;
    for v in 'a'..='z' {
        let polymer: String = polymer
            .chars()
            .filter(|c| *c != v && *c != v.to_ascii_uppercase())
            .collect();
        let polymer = decompose(polymer.as_str())?;
        println!("Test({}) decomposed polymer has length: {}", v, polymer.len());

        min = match min {
            Some(p) if p.len() < polymer.len() => Some(p),
            _ => Some(polymer),
        }
    }

    Ok(min.unwrap())
}
