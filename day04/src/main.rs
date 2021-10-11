use anyhow::{anyhow, bail, Context, Result};
use chrono::{NaiveDateTime, Timelike};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, Read},
    str::FromStr,
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let log = read_log(&input)?;
    let sleep_map = sleep_map(&log)?;

    let sleep_factor = sleep_factor1(&sleep_map)?;
    println!("Answer to part 1: {:?}", sleep_factor);

    let sleep_factor = sleep_factor2(&sleep_map)?;
    println!("Answer to part 2: {:?}", sleep_factor);

    Ok(())
}

#[derive(Debug, PartialEq)]
struct LogEntry {
    time: NaiveDateTime,
    event: Event,
}

#[derive(Debug, PartialEq)]
enum Event {
    BeginShift { guard: Guard },
    FallAsleep,
    WakeUp,
}

type Guard = u32;

type SleepMap = HashMap<Guard, [u32; 60]>;

impl FromStr for LogEntry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE_LOG: Regex = Regex::new(r"^\[(?P<time>[^\]]+)\]\s+(?P<msg>.*)").unwrap();
            static ref RE_BEGIN: Regex =
                Regex::new(r"^Guard #(?P<guard_id>\d+) begins shift").unwrap();
        }
        let caps = RE_LOG
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid log entry: {}", s))?;

        let time = NaiveDateTime::parse_from_str(&caps["time"], "%Y-%m-%d %H:%M")?;
        let msg = &caps["msg"];
        let event = match RE_BEGIN.captures(msg) {
            Some(caps) => Event::BeginShift {
                guard: caps["guard_id"].parse()?,
            },
            None => match msg {
                "falls asleep" => Event::FallAsleep,
                "wakes up" => Event::WakeUp,
                _ => bail!("Invalid log entry message: {}", msg),
            },
        };

        Ok(LogEntry { time, event })
    }
}

fn read_log(input: &str) -> Result<Vec<LogEntry>> {
    let mut lines: Vec<&str> = input.lines().collect();
    lines.sort_unstable();
    lines.iter().map(|line| line.parse()).collect()
}

fn sleep_map(log: &[LogEntry]) -> Result<SleepMap> {
    let mut sleep_map = SleepMap::new();
    let mut active_guard = None;
    let mut sleep_start = None;

    for entry in log {
        match entry.event {
            Event::BeginShift { guard } => {
                if sleep_start.is_some() {
                    bail!("New guard cannot start if previous is asleep: {:?}", entry);
                }
                active_guard = Some(guard);
            }
            Event::FallAsleep => {
                if active_guard.is_none() {
                    bail!("No guard on duty to fall asleep: {:?}", entry);
                }
                if sleep_start.is_some() {
                    bail!("Cannot fall asleep if already asleep: {:?}", entry);
                }
                if entry.time.hour() != 0 {
                    bail!("Guards can only sleep during midnight hour: {:?}", entry);
                }
                sleep_start = Some(&entry.time);
            }
            Event::WakeUp => {
                if entry.time.hour() != 0 {
                    bail!("Guards can only sleep during midnight hour: {:?}", entry);
                }
                match (active_guard, sleep_start) {
                    (None, _) => bail!("No guard on duty to wake up: {:?}", entry),
                    (_, None) => bail!("Cannot wake up if not asleep: {:?}", entry),
                    (Some(active_guard), Some(sleep_start)) => {
                        let tally = sleep_map.entry(active_guard).or_insert([0u32; 60]);
                        for min in sleep_start.minute()..entry.time.minute() {
                            tally[min as usize] += 1;
                        }
                    }
                }

                sleep_start = None;
            }
        }
    }

    Ok(sleep_map)
}

fn max_with_index<T: PartialOrd>(s: &[T]) -> Option<(usize, &T)> {
    s.iter().enumerate().fold(None, |acc, (i, v)| match acc {
        None => Some((i, v)),
        Some((_, zv)) if v > zv => Some((i, v)),
        _ => acc,
    })
}

fn sleep_factor1(sleep_map: &SleepMap) -> Result<u32> {
    let mut found_guard = None;
    let mut found_ind = 0;
    let mut found_mins = 0;
    for (guard, tally) in sleep_map {
        let mins: u32 = tally.iter().sum();
        if mins > found_mins {
            found_guard = Some(guard);
            found_mins = mins;
            found_ind = max_with_index(tally).map(|r| r.0).unwrap();
        }
    }

    if let Some(found_guard_id) = found_guard {
        Ok(found_guard_id * found_ind as u32)
    } else {
        Err(anyhow!("No guards slept"))
    }
}

fn sleep_factor2(sleep_map: &SleepMap) -> Result<u32> {
    let mut found_guard = None;
    let mut found_ind = 0;
    let mut found_mins = 0;
    for (guard, tally) in sleep_map {
        let (ind, &mins) = max_with_index(tally).unwrap();
        if mins > found_mins {
            found_guard = Some(guard);
            found_ind = ind;
            found_mins = mins;
        }
    }

    found_guard
        .context("No guards slept")
        .map(|guard| guard * found_ind as u32)
}
