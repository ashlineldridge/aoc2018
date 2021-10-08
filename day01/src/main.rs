use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Seek},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn calibrate1(reader: &mut BufReader<File>) -> Result<i32> {
    let mut freq = 0;
    for res in reader.lines() {
        let line = res?;
        let delta = line.parse::<i32>()?;
        freq += delta;
    }

    Ok(freq)
}

fn calibrate2(reader: &mut BufReader<File>, seen_count: u32) -> Result<i32> {
    let mut map: HashMap<i32, u32> = HashMap::new();
    let mut freq = 0;
    loop {
        for res in reader.lines() {
            let line = res?;
            let delta = line.parse::<i32>()?;
            freq += delta;
            let seen = map.get(&freq).unwrap_or(&0) + 1;
            if seen == seen_count {
                return Ok(freq);
            }

            map.insert(freq, seen);
        }

        reader.rewind()?;
    }
}

fn main() -> Result<()> {
    let file = File::open("input/input.txt")?;
    let mut reader = BufReader::new(file);

    let freq = calibrate1(&mut reader)?;
    println!("Calibration frequency 1: {}", freq);

    reader.rewind()?;
    let freq = calibrate2(&mut reader, 2)?;
    println!("Calibration frequency 2: {}", freq);

    Ok(())
}
