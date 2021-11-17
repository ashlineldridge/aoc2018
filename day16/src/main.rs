use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    io::{self, Read},
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    part1(input)?;

    Ok(())
}

fn part1(input: String) -> Result<()> {
    let samples = read_samples(&input)?;
    let mut three_or_more = 0;

    for sample in samples {
        let ops = decode_sample(&sample);
        if ops.len() >= 3 {
            three_or_more += 1;
        }
    }

    println!("Part 1 answer: {}", three_or_more);

    Ok(())
}

type RegVal = u16;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Registers([RegVal; Registers::TOTAL_REGISTERS]);

impl Registers {
    const TOTAL_REGISTERS: usize = 4;

    fn get_reg<T: Into<usize> + Copy>(&self, index: T) -> RegVal {
        Self::assert_index(index.into());
        self.0[index.into()]
    }

    fn set_reg<T: Into<usize> + Copy>(&mut self, index: T, value: RegVal) {
        Self::assert_index(index.into());
        self.0[index.into()] = value;
    }

    fn assert_index(index: usize) {
        assert!(
            index < Self::TOTAL_REGISTERS,
            "invalid register index: {}",
            index
        );
    }
}

impl FromStr for Registers {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = s
            .split_whitespace()
            .map(|v| v.parse::<RegVal>().context("bad register value"))
            .collect::<Result<Vec<_>>>()?;

        Ok(Registers(vec.try_into().unwrap()))
    }
}

#[derive(Clone, Debug)]
struct Instruction {
    op: Op,
    a: RegVal,
    b: RegVal,
    c: RegVal,
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
enum Op {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

impl Instruction {
    fn exec(&self, registers: &mut Registers) {
        match self.op {
            Op::Addr => self.addr(registers),
            Op::Addi => self.addi(registers),
            Op::Mulr => self.mulr(registers),
            Op::Muli => self.muli(registers),
            Op::Banr => self.banr(registers),
            Op::Bani => self.bani(registers),
            Op::Borr => self.borr(registers),
            Op::Bori => self.bori(registers),
            Op::Setr => self.setr(registers),
            Op::Seti => self.seti(registers),
            Op::Gtir => self.gtir(registers),
            Op::Gtri => self.gtri(registers),
            Op::Gtrr => self.gtrr(registers),
            Op::Eqir => self.eqir(registers),
            Op::Eqri => self.eqri(registers),
            Op::Eqrr => self.eqrr(registers),
        }
    }

    fn addr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) + registers.get_reg(self.b);
        registers.set_reg(self.c, res);
    }

    fn addi(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) + self.b;
        registers.set_reg(self.c, res);
    }

    fn mulr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) * registers.get_reg(self.b);
        registers.set_reg(self.c, res);
    }

    fn muli(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) * self.b;
        registers.set_reg(self.c, res);
    }

    fn banr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) & registers.get_reg(self.b);
        registers.set_reg(self.c, res);
    }

    fn bani(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) & self.b;
        registers.set_reg(self.c, res);
    }

    fn borr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) | registers.get_reg(self.b);
        registers.set_reg(self.c, res);
    }

    fn bori(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a) | self.b;
        registers.set_reg(self.c, res);
    }

    fn setr(&self, registers: &mut Registers) {
        registers.set_reg(self.c, registers.get_reg(self.a));
    }

    fn seti(&self, registers: &mut Registers) {
        registers.set_reg(self.c, self.a);
    }

    fn gtir(&self, registers: &mut Registers) {
        let res = (self.a > registers.get_reg(self.b)) as RegVal;
        registers.set_reg(self.c, res);
    }

    fn gtri(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a) > self.b) as RegVal;
        registers.set_reg(self.c, res);
    }

    fn gtrr(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a) > registers.get_reg(self.b)) as RegVal;
        registers.set_reg(self.c, res);
    }

    fn eqir(&self, registers: &mut Registers) {
        let res = (self.a == registers.get_reg(self.b)) as RegVal;
        registers.set_reg(self.c, res);
    }

    fn eqri(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a) == self.b) as RegVal;
        registers.set_reg(self.c, res);
    }

    fn eqrr(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a) == registers.get_reg(self.b)) as RegVal;
        registers.set_reg(self.c, res);
    }
}

struct Sample {
    before: Registers,
    after: Registers,
    a: RegVal,
    b: RegVal,
    c: RegVal,
}

impl FromStr for Sample {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)^
                Before:\s+\[(?P<before>\d+,\ \d+,\ \d+,\ \d+)\]\s+
                (?P<instr>\d+\ \d+\ \d+\ \d+)\s+
                After:\s+\[(?P<after>\d+,\ \d+,\ \d+,\ \d+)\]"
            )
            .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid sample: {}", s))?;

        let instr = caps["instr"].parse::<Registers>()?;

        Ok(Self {
            before: caps["before"].replace(",", "").parse()?,
            after: caps["after"].replace(",", "").parse()?,
            a: instr.get_reg(1_usize),
            b: instr.get_reg(2_usize),
            c: instr.get_reg(3_usize),
        })
    }
}

fn read_samples(input: &str) -> Result<Vec<Sample>> {
    let lines = input
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let mut samples = vec![];
    for chunk in lines.chunks(3) {
        let sample = chunk.join(" ").parse()?;
        samples.push(sample);
    }

    Ok(samples)
}

fn decode_sample(sample: &Sample) -> Vec<Op> {
    let mut compatible_ops = vec![];
    for op in Op::iter() {
        let instr = Instruction {
            op,
            a: sample.a,
            b: sample.b,
            c: sample.c,
        };
        let mut test = sample.before.clone();
        instr.exec(&mut test);
        if test == sample.after {
            compatible_ops.push(op);
        }
    }

    compatible_ops
}
