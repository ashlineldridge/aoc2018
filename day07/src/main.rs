use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::{
    io::{self, Read},
    str::FromStr,
};

const WORKER_COUNT: u32 = 5;
const WORK_BASE_TIME_SECONDS: u32 = 60;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let deps = read_deps(&input)?;
    let answer = part1(&deps)?;
    println!("Answer to part 1: {}", answer);

    let mut worker_pool = WorkerPool::new(WORKER_COUNT);
    let answer = part2(&deps, &mut worker_pool)?;
    println!("Answer to part 2: {} seconds", answer);

    Ok(())
}

fn part1(deps: &DepsByStep) -> Result<String> {
    let mut completed_steps = HashSet::new();
    let mut available_steps = deps
        .iter()
        .filter_map(|(s, v)| if v.is_empty() { Some(*s) } else { None })
        .collect::<HashSet<Step>>();

    let mut result = String::from("");
    while !available_steps.is_empty() {
        let chosen = *available_steps.iter().max_by(|s1, s2| s2.cmp(s1)).unwrap();
        available_steps.remove(&chosen);
        completed_steps.insert(chosen);
        result += chosen.to_string().as_str();

        let unlocked_steps = deps
            .iter()
            .filter_map(|(s, v)| {
                if !completed_steps.contains(s) && v.is_subset(&completed_steps) {
                    Some(*s)
                } else {
                    None
                }
            })
            .collect::<HashSet<Step>>();
        available_steps.extend(unlocked_steps);
    }

    Ok(result)
}

fn part2(deps: &DepsByStep, worker_pool: &mut WorkerPool) -> Result<u32> {
    let mut completed_steps = HashSet::new();
    let mut in_flight_steps = HashSet::new();
    let mut available_steps = deps
        .iter()
        .filter_map(|(s, v)| if v.is_empty() { Some(*s) } else { None })
        .collect::<HashSet<Step>>();

    let mut result = 0;
    loop {
        if completed_steps.len() == deps.len() {
            break;
        }

        while !available_steps.is_empty() && worker_pool.is_ready() {
            let chosen = *available_steps.iter().max_by(|s1, s2| s2.cmp(s1)).unwrap();
            let duration = WORK_BASE_TIME_SECONDS + (chosen as u8 - b'A') as u32 + 1;
            worker_pool.assign(&Task::new(chosen, duration));
            in_flight_steps.insert(chosen);
            available_steps.remove(&chosen);
        }

        loop {
            worker_pool.tick();
            result += 1;
            let batch_completed = worker_pool.receive();
            if !batch_completed.is_empty() {
                completed_steps.extend(&batch_completed);
                in_flight_steps = in_flight_steps
                    .difference(&batch_completed)
                    .cloned()
                    .collect();

                let unlocked_steps = deps
                    .iter()
                    .filter_map(|(s, v)| {
                        if v.is_subset(&completed_steps)
                            && !completed_steps.contains(s)
                            && !in_flight_steps.contains(s)
                        {
                            Some(*s)
                        } else {
                            None
                        }
                    })
                    .collect::<HashSet<Step>>();
                available_steps.extend(unlocked_steps);
                break;
            }
        }
    }

    Ok(result)
}

type Step = char;

type DepsByStep = HashMap<Step, HashSet<Step>>;

#[derive(Eq, Hash, PartialEq)]
struct DepPair(Step, Step);

impl FromStr for DepPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^Step (?P<s1>[A-Z]) must be finished before step (?P<s2>[A-Z])")
                    .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid line: {}", s))?;

        Ok(DepPair(caps["s2"].parse()?, caps["s1"].parse()?))
    }
}

struct WorkerPool {
    workers: Vec<Worker>,
}

impl WorkerPool {
    fn new(size: u32) -> WorkerPool {
        let mut worker_pool = WorkerPool { workers: vec![] };
        for _ in 0..size {
            worker_pool.workers.push(Worker::new());
        }
        worker_pool
    }

    fn assign(&mut self, task: &Task) {
        for worker in &mut self.workers {
            if worker.is_ready() {
                worker.assign(task);
                return;
            }
        }

        panic!("No available worker");
    }

    fn tick(&mut self) {
        for worker in &mut self.workers {
            worker.tick();
        }
    }

    fn receive(&mut self) -> HashSet<Step> {
        let mut completed = HashSet::new();
        for worker in &mut self.workers {
            if let Some(step) = worker.receive() {
                completed.insert(step);
            }
        }

        completed
    }

    fn is_ready(&self) -> bool {
        for worker in &self.workers {
            if worker.is_ready() {
                return true;
            }
        }
        false
    }
}

struct Worker {
    task: Option<Task>,
}

impl Worker {
    fn new() -> Worker {
        Worker { task: None }
    }

    fn assign(&mut self, task: &Task) {
        if self.task.is_some() {
            panic!("Worker has incomplete work");
        }
        self.task = Some(task.clone());
    }

    fn receive(&mut self) -> Option<Step> {
        match &self.task {
            Some(task) if task.is_complete() => self.task.take().map(|u| u.step),
            _ => None,
        }
    }

    fn tick(&mut self) {
        if let Some(task) = self.task.as_mut() {
            task.tick();
        }
    }

    fn is_ready(&self) -> bool {
        self.task.is_none()
    }
}

#[derive(Clone)]
struct Task {
    step: Step,
    duration: u32,
}

impl Task {
    fn new(step: Step, duration: u32) -> Task {
        Task { step, duration }
    }

    fn tick(&mut self) {
        self.duration = self.duration.saturating_sub(1);
    }

    fn is_complete(&self) -> bool {
        self.duration == 0
    }
}

fn read_deps(input: &str) -> Result<DepsByStep> {
    let pairs: Result<HashSet<DepPair>> = input.lines().map(|v| v.parse()).collect();
    let pairs = pairs?;

    let mut deps = DepsByStep::new();
    for pair in pairs {
        deps.entry(pair.0).or_default().insert(pair.1);
        deps.entry(pair.1).or_default();
    }

    Ok(deps)
}
