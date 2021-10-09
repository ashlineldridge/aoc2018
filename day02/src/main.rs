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
