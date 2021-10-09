use std::{
    collections::HashSet,
    io::{self, Read},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let freq = calibrate1(&input)?;
    println!("Calibration frequency 1: {}", freq);

    let freq = calibrate2(&input)?;
    println!("Calibration frequency 2: {}", freq);

    Ok(())
}

fn calibrate1(input: &str) -> Result<i32> {
    let mut freq = 0;
    for line in input.lines() {
        let delta = line.parse::<i32>()?;
        freq += delta;
    }

    Ok(freq)
}

fn calibrate2(input: &str) -> Result<i32> {
    let mut freq = 0;
    let mut seen = HashSet::new();
    seen.insert(freq);

    loop {
        for line in input.lines() {
            let delta = line.parse::<i32>()?;
            freq += delta;
            if seen.contains(&freq) {
                return Ok(freq);
            }

            seen.insert(freq);
        }
    }
}
