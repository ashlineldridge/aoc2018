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
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^\[(?P<time>[^\]]+)\]\s+
                (?:Guard\ \#(?P<id>\d+)\ begins\ shift|(?P<sleep>.*))$
            "
            )
            .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid log entry: {}", s))?;
        let time = NaiveDateTime::parse_from_str(&caps["time"], "%Y-%m-%d %H:%M")?;
        let event = match caps.name("id") {
            Some(m) => Event::BeginShift {
                guard: m.as_str().parse()?,
            },
            None => match &caps["sleep"] {
                "falls asleep" => Event::FallAsleep,
                "wakes up" => Event::WakeUp,
                _ => bail!("Invalid log entry message: {}", s),
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

fn sleep_factor1(sleep_map: &SleepMap) -> Result<u32> {
    sleep_factor(sleep_map, |tally| tally.iter().sum())
}

fn sleep_factor2(sleep_map: &SleepMap) -> Result<u32> {
    sleep_factor(sleep_map, |tally| *tally.iter().max().unwrap())
}

fn sleep_factor<F>(sleep_map: &SleepMap, max: F) -> Result<u32>
where
    F: Fn(&[u32; 60]) -> u32,
{
    let (guard, tally) = sleep_map
        .iter()
        .max_by_key(|&(_, tally)| max(tally))
        .context("Empty log")?;

    let (minute, _) = tally
        .iter()
        .enumerate()
        .max_by_key(|&(_, count)| *count)
        .context("Invalid sleep tally")?;

    Ok(guard * minute as u32)
}
