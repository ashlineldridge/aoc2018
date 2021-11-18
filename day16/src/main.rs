use anyhow::{anyhow, ensure, Context, Result};
use enum_iterator::IntoEnumIterator;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let sections = input.split("\n\n\n\n").collect::<Vec<_>>();
    ensure!(sections.len() == 2, "invalid program input");

    let training_data = read_samples(sections[0])?;
    let program_data = read_registers(sections[1])?;

    part1(&training_data)?;
    part2(&training_data, &program_data)?;

    Ok(())
}

fn part1(samples: &[Sample]) -> Result<()> {
    let mut three_or_more = 0;
    for sample in samples {
        let results = Machine::test_sample(sample);
        if results.len() >= 3 {
            three_or_more += 1;
        }
    }

    println!("Part 1 answer: {}", three_or_more);

    Ok(())
}

fn part2(training_data: &[Sample], program_data: &[Registers]) -> Result<()> {
    let mut machine = Machine::build(training_data)?;

    for raw in program_data {
        let instr = machine.decode_instruction(raw)?;
        machine.execute_instruction(&instr);
    }

    println!("Part 2 answer: {}", machine.registers.get_reg(0_usize));

    Ok(())
}

type RegVal = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Registers([RegVal; Registers::TOTAL_REGISTERS]);

impl Registers {
    const TOTAL_REGISTERS: usize = 4;

    fn get_reg(&self, index: usize) -> RegVal {
        Self::assert_index(index);
        self.0[index]
    }

    fn set_reg(&mut self, index: usize, value: RegVal) {
        Self::assert_index(index);
        self.0[index] = value;
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

impl Default for Registers {
    fn default() -> Self {
        Self([0; Registers::TOTAL_REGISTERS])
    }
}

#[derive(Clone, Debug)]
struct Instruction {
    op: Op,
    a: RegVal,
    b: RegVal,
    c: RegVal,
}

#[derive(Clone, Copy, Debug, IntoEnumIterator, Eq, Hash, PartialEq)]
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
    fn execute(&self, registers: &mut Registers) {
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
        let res = registers.get_reg(self.a as usize) + registers.get_reg(self.b as usize);
        registers.set_reg(self.c as usize, res);
    }

    fn addi(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) + self.b;
        registers.set_reg(self.c as usize, res);
    }

    fn mulr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) * registers.get_reg(self.b as usize);
        registers.set_reg(self.c as usize, res);
    }

    fn muli(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) * self.b;
        registers.set_reg(self.c as usize, res);
    }

    fn banr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) & registers.get_reg(self.b as usize);
        registers.set_reg(self.c as usize, res);
    }

    fn bani(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) & self.b;
        registers.set_reg(self.c as usize, res);
    }

    fn borr(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) | registers.get_reg(self.b as usize);
        registers.set_reg(self.c as usize, res);
    }

    fn bori(&self, registers: &mut Registers) {
        let res = registers.get_reg(self.a as usize) | self.b;
        registers.set_reg(self.c as usize, res);
    }

    fn setr(&self, registers: &mut Registers) {
        registers.set_reg(self.c as usize, registers.get_reg(self.a as usize));
    }

    fn seti(&self, registers: &mut Registers) {
        registers.set_reg(self.c as usize, self.a);
    }

    fn gtir(&self, registers: &mut Registers) {
        let res = (self.a > registers.get_reg(self.b as usize)) as RegVal;
        registers.set_reg(self.c as usize, res);
    }

    fn gtri(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a as usize) > self.b) as RegVal;
        registers.set_reg(self.c as usize, res);
    }

    fn gtrr(&self, registers: &mut Registers) {
        let res =
            (registers.get_reg(self.a as usize) > registers.get_reg(self.b as usize)) as RegVal;
        registers.set_reg(self.c as usize, res);
    }

    fn eqir(&self, registers: &mut Registers) {
        let res = (self.a == registers.get_reg(self.b as usize)) as RegVal;
        registers.set_reg(self.c as usize, res);
    }

    fn eqri(&self, registers: &mut Registers) {
        let res = (registers.get_reg(self.a as usize) == self.b) as RegVal;
        registers.set_reg(self.c as usize, res);
    }

    fn eqrr(&self, registers: &mut Registers) {
        let res =
            (registers.get_reg(self.a as usize) == registers.get_reg(self.b as usize)) as RegVal;
        registers.set_reg(self.c as usize, res);
    }
}

struct Sample {
    before: Registers,
    after: Registers,
    instr: Registers,
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

        Ok(Self {
            before: caps["before"].replace(",", "").parse()?,
            after: caps["after"].replace(",", "").parse()?,
            instr: caps["instr"].parse()?,
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

fn read_registers(input: &str) -> Result<Vec<Registers>> {
    input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>>>()
}

struct Machine {
    registers: Registers,
    op_mapping: HashMap<RegVal, Op>,
}

impl Machine {
    /// Attempts to build a machine based on the supplied sample data.
    ///
    /// This will return an `Ok` if the sample data is complete. Sample data is
    /// considered complete when a single instruction can be found for each opcode.
    fn build(samples: &[Sample]) -> Result<Machine> {
        // Calculate the divergent ops - i.e., the opcodes for which there are one or
        // or more possible ops based on evaluating each sample in isolation.
        let mut divergent_ops: HashMap<RegVal, HashSet<Op>> = HashMap::new();
        for sample in samples {
            // Get the set of possible ops for the current sample.
            let sample_possible_ops = Self::test_sample(sample);

            // Update the existing possible ops for the sample's opcode to be equal
            // to the intersection of the set derived from the current sample and the
            // existing set. Whatever op we finally decide on must be able to meet
            // the requirements of every sample that involves that opcode.
            let possible_ops = divergent_ops
                .entry(sample.instr.get_reg(0))
                .or_insert_with(|| sample_possible_ops.clone());
            *possible_ops = possible_ops
                .intersection(&sample_possible_ops)
                .cloned()
                .collect::<HashSet<_>>();
        }

        // At this point, the set of all op possibilities must include an entry for
        // every op otherwise we won't be able to derive the full instruction set.
        ensure!(
            divergent_ops.len() == Op::VARIANT_COUNT,
            "incomplete training data"
        );

        // We'll repeatedly sweep across the op possibilites gathering
        let mut convergent_ops: HashMap<RegVal, Op> = HashMap::new();
        loop {
            // Calculate the next set of opcodes that have converged onto a single op.
            let next_converged = divergent_ops
                .iter()
                .filter_map(|(opcode, ops)| {
                    if ops.len() == 1 {
                        Some((*opcode, *ops.iter().next().unwrap()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            ensure!(!next_converged.is_empty(), "could not converge sample data");

            // Add each newly converged opcode/op entry into the convergent map and remove
            // it from the divergent map.
            for (opcode, op) in next_converged {
                convergent_ops.insert(opcode, op);
                divergent_ops.remove(&opcode);
            }

            // If the set of convergent ops is complete we're done.
            let convergent_ops = convergent_ops.values().cloned().collect::<HashSet<_>>();
            if convergent_ops.len() == Op::VARIANT_COUNT {
                break;
            }

            // For each divergent op, remove any of the convergent ops from its set of possibilities.
            for (_, ops) in divergent_ops.iter_mut() {
                *ops = ops
                    .difference(&convergent_ops)
                    .cloned()
                    .collect::<HashSet<_>>();
            }
        }

        Ok(Machine {
            registers: Default::default(),
            op_mapping: convergent_ops,
        })
    }

    fn decode_instruction(&self, raw: &Registers) -> Result<Instruction> {
        let opcode = raw.get_reg(0);
        let op = *self
            .op_mapping
            .get(&opcode)
            .context(format!("invalid opcode: {}", opcode))?;

        Ok(Instruction {
            op,
            a: raw.get_reg(1),
            b: raw.get_reg(2),
            c: raw.get_reg(3),
        })
    }

    fn execute_instruction(&mut self, instruction: &Instruction) {
        instruction.execute(&mut self.registers);
    }

    fn test_sample(sample: &Sample) -> HashSet<Op> {
        let mut compatible_ops = HashSet::new();
        for op in Op::into_enum_iter() {
            let instr = Instruction {
                op,
                a: sample.instr.get_reg(1),
                b: sample.instr.get_reg(2),
                c: sample.instr.get_reg(3),
            };
            let mut test = sample.before.clone();
            instr.execute(&mut test);
            if test == sample.after {
                compatible_ops.insert(op);
            }
        }

        compatible_ops
    }
}
