use anyhow::{anyhow, Result};
use core::time;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashSet,
    fmt::Display,
    io::{self, Read},
    str::FromStr,
    thread,
};

const MAX_PLOT_AREA: i32 = 3000;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let points = input
        .lines()
        .map(|v| v.parse())
        .collect::<Result<Vec<Point>>>()?;

    let mut plot = Plot::new(points);
    run_simulation(&mut plot);

    Ok(())
}

fn run_simulation(plot: &mut Plot) {
    let mut seconds = 0;
    while plot.area() > MAX_PLOT_AREA {
        plot.advance();
        seconds += 1;
    }

    while plot.area() <= MAX_PLOT_AREA {
        println!("Simulated time: {} seconds", seconds);
        println!("{}", plot);

        plot.advance();
        seconds += 1;

        thread::sleep(time::Duration::from_millis(300));
    }
}

#[derive(Clone, Debug)]
struct Point {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

impl Point {
    fn advance(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }
}

struct Plot {
    points: Vec<Point>,
}

impl Plot {
    fn new(points: Vec<Point>) -> Plot {
        Plot { points }
    }

    fn advance(&mut self) {
        for p in &mut self.points {
            p.advance();
        }
    }

    fn window(&self) -> Option<Window> {
        let mut window = None;
        for p in &self.points {
            match &mut window {
                None => window = Some(Window::new(p.x, p.x, p.y, p.y)),
                Some(window) => {
                    window.x_min = window.x_min.min(p.x);
                    window.x_max = window.x_max.max(p.x);
                    window.y_min = window.y_min.min(p.y);
                    window.y_max = window.y_max.max(p.y);
                }
            }
        }

        window
    }

    fn area(&self) -> i32 {
        self.window().map(|w| w.area()).unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
struct Window {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
}

impl Window {
    fn new(x_min: i32, x_max: i32, y_min: i32, y_max: i32) -> Window {
        Window {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    fn width(&self) -> i32 {
        self.x_max.abs() - self.x_min.abs()
    }

    fn height(&self) -> i32 {
        self.y_max.abs() - self.y_min.abs()
    }

    fn area(&self) -> i32 {
        self.width() * self.height()
    }
}

impl Display for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coords = self
            .points
            .iter()
            .map(|p| (p.x, p.y))
            .collect::<HashSet<(i32, i32)>>();

        let mut out = String::from("");
        if let Some(window) = self.window() {
            for y in window.y_min..=window.y_max {
                for x in window.x_min..=window.x_max {
                    if coords.contains(&(x, y)) {
                        out += "#";
                    } else {
                        out += ".";
                    }
                }

                out += "\n";
            }
        }

        f.write_str(out.as_str())?;

        Ok(())
    }
}

impl FromStr for Point {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)^
                position=<\s*(?P<px>-?\d+),\s*(?P<py>-?\d+)>\s+
                velocity=<\s*(?P<vx>-?\d+),\s*(?P<vy>-?\d+)>$"
            )
            .unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid line: {}", s))?;

        Ok(Point {
            x: caps["px"].parse()?,
            y: caps["py"].parse()?,
            vx: caps["vx"].parse()?,
            vy: caps["vy"].parse()?,
        })
    }
}
