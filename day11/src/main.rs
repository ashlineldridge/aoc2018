use anyhow::Result;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let serial = input.trim().parse::<usize>()?;
    let fuel_grid = FuelGrid::new(300, 300, serial);

    part1(&fuel_grid);
    part2(&fuel_grid);

    Ok(())
}

fn part1(fuel_grid: &FuelGrid) {
    let cell_group = find_best_cell_group(fuel_grid, 3).unwrap();
    println!(
        "Part 1: Max power is {} at ({}, {})",
        cell_group.total_power(),
        cell_group.x,
        cell_group.y
    );
}

fn part2(fuel_grid: &FuelGrid) {
    let mut best_cell_group: Option<FuelCellGroup> = None;
    for size in 1..=fuel_grid.width {
        let cell_group = find_best_cell_group(fuel_grid, size).unwrap();
        println!("Using group size {} max power is {}", size, cell_group.total_power());
        best_cell_group = match &best_cell_group {
            Some(g) if g.total_power() > cell_group.total_power() => best_cell_group,
            _ => Some(cell_group),
        };
    }

    let best_cell_group = best_cell_group.unwrap();
    println!(
        "Part 2: Max power is {} at ({}, {}) using group size {}",
        best_cell_group.total_power(),
        best_cell_group.x,
        best_cell_group.y,
        best_cell_group.group_width,
    );
}

fn find_best_cell_group(fuel_grid: &FuelGrid, size: usize) -> Option<FuelCellGroup> {
    fuel_grid
        .cell_groups(size, size)
        .max_by_key(|g| g.total_power())
}

struct FuelGrid {
    width: usize,
    height: usize,
    serial: usize,
}

impl FuelGrid {
    fn new(width: usize, height: usize, serial: usize) -> FuelGrid {
        FuelGrid {
            width,
            height,
            serial,
        }
    }

    fn cell_power(&self, x: usize, y: usize) -> Option<i32> {
        if !(1..=self.width).contains(&x) || !(1..=self.height).contains(&y) {
            return None;
        }

        let rack_id = x + 10;
        let power = (rack_id * y + self.serial) * rack_id;
        let power = ((power % 1000) / 100) as i32 - 5;

        Some(power)
    }

    fn cell_groups(&self, group_width: usize, group_height: usize) -> FuelGridIter {
        FuelGridIter::new(self, group_width, group_height)
    }
}

struct FuelGridIter<'a> {
    fuel_grid: &'a FuelGrid,
    group_width: usize,
    group_height: usize,
    curr_group: FuelCellGroup<'a>,
}

impl<'a> FuelGridIter<'a> {
    fn new(fuel_grid: &FuelGrid, group_width: usize, group_height: usize) -> FuelGridIter {
        FuelGridIter {
            fuel_grid,
            group_width,
            group_height,
            curr_group: FuelCellGroup {
                fuel_grid,
                x: 1,
                y: 1,
                group_width,
                group_height,
            },
        }
    }
}

impl<'a> Iterator for FuelGridIter<'a> {
    type Item = FuelCellGroup<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_group.y + self.group_height > self.fuel_grid.height + 1 {
            return None;
        }

        let res = self.curr_group.clone();

        self.curr_group.x += 1;
        if self.curr_group.x + self.group_width > self.fuel_grid.width + 1 {
            self.curr_group.x = 1;
            self.curr_group.y += 1;
        }

        Some(res)
    }
}

#[derive(Clone)]
struct FuelCellGroup<'a> {
    fuel_grid: &'a FuelGrid,
    x: usize,
    y: usize,
    group_width: usize,
    group_height: usize,
}

impl<'a> FuelCellGroup<'a> {
    fn total_power(&self) -> i32 {
        let mut power = 0;
        for x in self.x..self.x + self.group_width {
            for y in self.y..self.y + self.group_height {
                power += self.fuel_grid.cell_power(x, y).unwrap();
            }
        }

        power
    }
}
