use anyhow::Result;
use std::io::{self, Read};

#[tokio::main]
async fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let serial = input.trim().parse::<usize>()?;
    let fuel_grid = FuelGrid::new(300, 300, serial);

    part1(fuel_grid.clone()).await;
    part2(fuel_grid.clone()).await;

    Ok(())
}

async fn part1(fuel_grid: FuelGrid) {
    let cell_group = find_best_cell_group(fuel_grid, 3).unwrap();
    println!(
        "Part 1: Max power is {} at ({}, {})",
        cell_group.total_power, cell_group.x, cell_group.y
    );
}

async fn part2(fuel_grid: FuelGrid) {
    let mut handles = vec![];
    for size in 1..=fuel_grid.width {
        let fuel_grid = fuel_grid.clone();
        let handle = tokio::task::spawn(async move {
            find_best_cell_group(fuel_grid, size)
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        results.push(result);
    }

    let cell_group = results.iter().max_by_key(|g| g.total_power).unwrap();

    println!(
        "Part 2: Max power is {} at ({}, {}) using group size {}",
        cell_group.total_power, cell_group.x, cell_group.y, cell_group.width,
    );
}

fn find_best_cell_group(fuel_grid: FuelGrid, size: usize) -> Option<FuelCellGroup> {
    fuel_grid
        .cell_groups(size, size)
        .max_by_key(|g| g.total_power)
}

#[derive(Clone)]
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
    width: usize,
    height: usize,
    curr_x: usize,
    curr_y: usize,
}

impl<'a> FuelGridIter<'a> {
    fn new(fuel_grid: &FuelGrid, width: usize, height: usize) -> FuelGridIter {
        FuelGridIter {
            fuel_grid,
            width,
            height,
            curr_x: 1,
            curr_y: 1,
        }
    }
}

impl<'a> Iterator for FuelGridIter<'a> {
    type Item = FuelCellGroup;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_y + self.height > self.fuel_grid.height + 1 {
            return None;
        }

        let mut cell_group_power = 0;
        for x in self.curr_x..self.curr_x + self.width {
            for y in self.curr_y..self.curr_y + self.height {
                cell_group_power += self.fuel_grid.cell_power(x, y).unwrap();
            }
        }

        let cell_group = FuelCellGroup {
            x: self.curr_x,
            y: self.curr_y,
            width: self.width,
            height: self.height,
            total_power: cell_group_power,
        };

        self.curr_x += 1;
        if self.curr_x + self.width > self.fuel_grid.width + 1 {
            self.curr_x = 1;
            self.curr_y += 1;
        }

        Some(cell_group)
    }
}

struct FuelCellGroup {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    total_power: i32,
}
