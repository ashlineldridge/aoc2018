#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::{Context, Result};
use std::fmt::Display;
use std::str::{self, Chars};
use std::{
    io::{self, Read},
    str::FromStr,
};

// The spread factor specifies the number of plant siblings that have an influence on
// the next generation of plants. The total rule pattern width = 2 * spread + 1 (i.e.,
// the siblings on either side of the current plant plus the current plant itself).
const SPREAD_FACTOR: u32 = 2;

const RULE_WIDTH: u32 = 2 * SPREAD_FACTOR + 1;

const TOTAL_RULES: usize = 2_u32.pow(RULE_WIDTH) as usize;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Even after optimizing this to use bit manipulation rather than hashsets, etc,
    // 50,000,000,000 iterations was still gonna take longer than I could be bothered
    // waiting around. Running the below numbers produces the following pattern:
    //
    // After 500 generations the answer is: 21684
    // After 5000 generations the answer is: 201684
    // After 50000 generations the answer is: 2001684
    //
    // So, for 5XXX generations, where XXX is some number of zeros greater than two, the
    // answer is 2YYY1684 where YYYY = the number of zeros in XXX - 2. The answer for
    // 50,000,000,000 generations is then 2,000,000,001,684.

    let nursery: Nursery = input.parse()?;
    run_sim(nursery.clone(), 500);
    run_sim(nursery.clone(), 5_000);
    run_sim(nursery, 50_000);

    Ok(())
}

fn run_sim(mut nursery: Nursery, count: usize) {
    for _ in 0..count {
        nursery.advance();
    }

    let sum = nursery.sum();
    println!("After {} generations the answer is: {}", count, sum);
}

#[derive(Clone)]
struct Nursery {
    plants: Vec<u8>,
    plant_zero_index: usize,
    rules: [u8; TOTAL_RULES],
}

impl Nursery {
    fn new(mut plants: Vec<u8>, rules: [u8; TOTAL_RULES]) -> Nursery {
        let mut plant_zero_index = 0;
        if *plants.first().unwrap_or(&0) != 0 {
            plants.insert(0, 0);
            plant_zero_index += 1;
        }

        Nursery { plants, plant_zero_index, rules }
    }

    fn sum(&self) -> i32 {
        let mut sum = 0;
        for (i, byte) in self.plants.iter().enumerate() {
            for bit in 0..u8::BITS {
                if byte & (0x01 << bit) != 0 {
                    let plant_id = (i as i32 - self.plant_zero_index as i32) * u8::BITS as i32
                        + (u8::BITS - bit - 1) as i32;
                    sum += plant_id;
                }
            }
        }

        sum
    }

    fn advance(&mut self) {
        if *self.plants.last().unwrap_or(&0) != 0 {
            self.plants.push(0);
        }

        let mut new_plants = Vec::with_capacity(self.plants.len() + 2);
        if *self.plants.first().unwrap_or(&0) != 0 {
            new_plants.push(0);
            self.plant_zero_index += 1;
        }

        let mut prev_byte = 0;
        for i in 0..self.plants.len() {
            let this_byte = self.plants[i];
            let next_byte = *self.plants.get(i + 1).unwrap_or(&0);
            let mut new_byte = 0;

            let byte_group = (prev_byte as u32) << (u8::BITS * 2)
                | (this_byte as u32) << u8::BITS
                | next_byte as u32;

            for bit in 0..u8::BITS {
                let bit_index = i as u32 * u8::BITS + bit;
                let max_bit_index = (i as u32 + 2) * u8::BITS - 1;
                let shift_by = max_bit_index - bit_index - SPREAD_FACTOR;
                let rule_id = (byte_group >> shift_by) as u8 & !(0xFF << RULE_WIDTH);
                let outcome = self.rules[rule_id as usize];

                if outcome > 0 {
                    new_byte |= 0x01 << (u8::BITS - bit - 1);
                }
            }

            new_plants.push(new_byte);
            prev_byte = this_byte;
        }

        self.plants = new_plants;
    }
}

impl Display for Nursery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::from("");
        for b in &self.plants {
            out += format!("{:08b}", b).as_str();
        }

        let pattern = out.replace("1", "#").replace("0", ".");

        f.write_str(pattern.as_str())
    }
}

impl FromStr for Nursery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const BAD_INPUT: &str = "Bad input";

        let lines = s.lines().collect::<Vec<&str>>();
        let (head, tail) = lines.split_first().context(BAD_INPUT)?;

        let pattern = head.split_whitespace().last().context(BAD_INPUT)?;
        let initial_plants = str_to_byte_vec(pattern)?;

        let rule_tuples = tail
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let splat = line.split_whitespace().collect::<Vec<_>>();
                let rule_pattern = str_to_byte(splat.first().context(BAD_INPUT)?)?;
                let rule_outcome = str_to_byte(splat.last().context(BAD_INPUT)?)?;
                Ok((rule_pattern, rule_outcome))
            })
            .collect::<Result<Vec<(u8, u8)>>>()?;

        let mut rules = [0; TOTAL_RULES];
        for r in rule_tuples {
            rules[r.0 as usize] = r.1;
        }

        Ok(Nursery::new(initial_plants, rules))
    }
}

fn str_to_byte_vec(pattern: &str) -> Result<Vec<u8>> {
    pattern
        .chars()
        .collect::<Vec<char>>()
        .chunks(u8::BITS as usize)
        .map(|b| {
            let pattern = b.iter().collect::<String>();
            let mut byte = str_to_byte(pattern.as_str())?;
            byte <<= u8::BITS - b.len() as u32;
            Ok(byte)
        })
        .collect::<Result<Vec<u8>>>()
}

fn str_to_byte(pattern: &str) -> Result<u8> {
    let byte = pattern.replace("#", "1").replace(".", "0");
    Ok(u8::from_str_radix(&byte, 2)?)
}
