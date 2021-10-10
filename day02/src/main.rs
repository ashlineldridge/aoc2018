use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let checksum = checksum(&input, vec![2, 3])?;
    println!("Checksum: {}", checksum);

    if let Some(common) = find_common(&input) {
        println!("Common: {}", common);
    } else {
        println!("Could not find common match");
    }

    Ok(())
}

fn checksum(input: &str, counts: Vec<u32>) -> Result<u32> {
    let mut counts: HashMap<&u32, u32> = counts.iter().zip((0..1).cycle()).collect();
    for line in input.lines() {
        let mut seen: HashMap<char, u32> = HashMap::new();
        for c in line.chars() {
            let count = seen.get(&c).unwrap_or(&0) + 1;
            seen.insert(c, count);
        }

        let values: HashSet<&u32> = seen.values().into_iter().collect();
        for (k, v) in counts.iter_mut() {
            if values.contains(*k) {
                *v += 1;
            }
        }
    }

    Ok(counts.values().product())
}

fn find_common(input: &str) -> Option<String> {
    let lines: Vec<&str> = input.lines().collect();
    for i in 0..lines.len() {
        for j in i + 1..lines.len() {
            let (li, lj) = (lines[i], lines[j]);
            let mut common = String::from("");
            for (ci, cj) in li.chars().zip(lj.chars()) {
                if ci == cj {
                    common += ci.to_string().as_str();
                }
            }

            if common.len() == li.len() - 1 {
                return Some(common);
            }
        }
    }

    None
}
