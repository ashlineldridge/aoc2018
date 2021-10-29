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
const SPREAD_FACTOR: usize = 2;

const TOTAL_RULES: usize = 2_usize.pow((2 * SPREAD_FACTOR + 1) as u32);

fn main() -> Result<()> {
    // let left_row: Vec<u8> = vec![0b01101011, 0b10101011, 0b01111010];
    // let right_row: Vec<u8> = vec![0b01101011, 0b10101011, 0b01111010];
    // let nursery = Nursery {
    //     negative_plants: left_row,
    //     positive_plants: right_row,
    //     rules: [0; TOTAL_RULES],
    // };

    // println!("{:08b}", nursery.plant_group(-17));

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut nursery: Nursery = input.parse()?;
    // println!("{}", nursery);
    run_sim(&mut nursery, 4);

    Ok(())
}

fn run_sim(nursery: &mut Nursery, count: usize) {
    println!("0: {}", nursery);
    for i in 0..count {
        nursery.advance();
        println!("{}: {}", i + 1, nursery);
    }

    let sum = nursery.sum();
    println!("After {} generations the answer is: {}", count, sum);
}

#[derive(Debug)]
struct Nursery {
    negative_plants: Vec<u8>,
    positive_plants: Vec<u8>,
    rules: [u8; TOTAL_RULES],
}

impl Nursery {
    fn advance(&mut self) {
        let mut next_negative_plants = vec![];
        for i in 0..self.negative_plants.len() {
            let mut byte = 0;
            for bit in 0..u8::BITS {
                let plant_id = -(i as i32 * u8::BITS as i32 + bit as i32);
                let plant_group = self.plant_group(plant_id);
                let outcome = self.rules[plant_group as usize];

                if outcome > 0 {
                    byte |= 0x01 << bit;
                }
                println!("ID: {}, Group: {:05b}, Outcome: {}, Byte {}: {:08b}", plant_id, plant_group, outcome, i, byte);

            }

            next_negative_plants.push(byte);
        }

        self.negative_plants = next_negative_plants;

        let mut next_positive_plants = vec![];
        for i in 0..self.positive_plants.len() {
            let mut byte = 0;
            for bit in 0..u8::BITS {
                let plant_id = i as i32 * u8::BITS as i32 + bit as i32;
                let plant_group = self.plant_group(plant_id);
                let outcome = self.rules[plant_group as usize];

                if outcome > 0 {
                    // println!("Shifting by: {} for {}", u8::BITS - bit - 1, bit);
                    byte |= 0x01 << (u8::BITS - bit - 1);
                }
                // println!("ID: {}, Group: {:05b}, Outcome: {}, Byte {}: {:08b}", plant_id, plant_group, outcome, i, byte);
            }

            next_positive_plants.push(byte);
        }

        self.positive_plants = next_positive_plants;
    }

    fn plant_group(&self, id: i32) -> u8 {
        let bit_range = (id - SPREAD_FACTOR as i32, id + SPREAD_FACTOR as i32);
        let byte_range = (bit_range.0 / u8::BITS as i32, bit_range.1 / u8::BITS as i32);

        // println!("plant_group: id: {}, bit_range: {:?}, byte_range: {:?}", id, bit_range, byte_range);

        let left_byte = if bit_range.0 < 0 {
            self.negative_plants
                .get(byte_range.0.abs() as usize)
                .unwrap_or(&0)
                .reverse_bits()
        } else {
            *self.positive_plants.get(byte_range.0 as usize).unwrap_or(&0)
        };

        let (right_byte, right_shift) = if bit_range.1 < 0 {
            (
                self.negative_plants
                    .get(byte_range.1.abs() as usize)
                    .unwrap_or(&0)
                    .reverse_bits(),
                bit_range.1.abs() as u32 % u8::BITS,
            )
        } else {
            (
                *self.positive_plants.get(byte_range.1 as usize).unwrap_or(&0),
                u8::BITS - (bit_range.1.abs() as u32 % u8::BITS) - 1,
            )
        };

        let mut plant_group = (left_byte as u16) << u8::BITS | right_byte as u16;
        plant_group >>= right_shift;
        plant_group &= 0xFF >> (u8::BITS as usize - (2 * SPREAD_FACTOR + 1));

        plant_group as u8
    }

    fn sum(&self) -> i32 {
        let mut sum = 0;
        for (i, b) in self.positive_plants.iter().enumerate() {
            let mut b = *b;
            for bit in 0..u8::BITS {
                if b & 0x01 == 1 {
                    sum += i as i32 * u8::BITS as i32 + bit as i32;
                }
                b >>= 1;
            }
        }

        for (i, b) in self.negative_plants.iter().enumerate() {
            let mut b = *b;
            for bit in 0..u8::BITS {
                if b & 0x01 == 1 {
                    sum += -(i as i32 * u8::BITS as i32 + bit as i32);
                }
                b >>= 1;
            }
        }

        sum
    }
}

impl Display for Nursery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::from("");

        for (i, b) in self.negative_plants.iter().enumerate().rev() {
            if i == 0 {
                out += format!("{:07b}", b >> 1).as_str();
            } else {
                out += format!("{:08b}", b).as_str();
            }
        }

        for b in &self.positive_plants {
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

        Ok(Nursery {
            negative_plants: vec![0],
            positive_plants: initial_plants,
            rules,
        })
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
    let byte = pattern
        .replace("#", "1")
        .replace(".", "0");
    Ok(u8::from_str_radix(&byte, 2)?)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn plant_group() {
        let left_row: Vec<u8> = vec![0b01101011, 0b10101011, 0b01111010];
        let right_row: Vec<u8> = vec![0b01101011, 0b10101011, 0b01111010];
        let nursery = Nursery {
            negative_plants: left_row,
            positive_plants: right_row,
            rules: [0; TOTAL_RULES],
        };

        println!("{:08b}", nursery.plant_group(-17));
        assert_eq!(nursery.plant_group(8), 0b11101);
        assert_eq!(nursery.plant_group(0), 0b11011);
        assert_eq!(nursery.plant_group(-9), 0b01011);
        assert_eq!(nursery.plant_group(-17), 0b11101);
    }
}
