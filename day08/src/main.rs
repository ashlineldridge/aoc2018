use anyhow::{bail, Context, Result};
use std::{
    collections::VecDeque,
    io::{self, Read},
};

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let values: Result<VecDeque<u32>, _> =
        input.trim().split(' ').map(|v| v.parse::<u32>()).collect();
    let mut values = values?;
    let root = build_tree(&mut values)?;

    println!("Answer to part 1 is {}", root.value1());
    println!("Answer to part 2 is {}", root.value2());

    Ok(())
}

struct Node {
    children: Vec<Node>,
    entries: Vec<u32>,
}

impl Node {
    fn new() -> Node {
        Node {
            children: vec![],
            entries: vec![],
        }
    }

    fn value1(&self) -> u32 {
        self.children.iter().fold(0, |acc, n| acc + n.value1()) + self.entries.iter().sum::<u32>()
    }

    fn value2(&self) -> u32 {
        if self.children.is_empty() {
            return self.entries.iter().sum();
        }

        let mut value = 0;
        for entry in &self.entries {
            if *entry > 0 {
                let index = entry.saturating_sub(1) as usize;
                value += self.children.get(index).map(|n| n.value2()).unwrap_or(0);
            }
        }

        value
    }
}

fn build_tree(values: &mut VecDeque<u32>) -> Result<Node> {
    let total_children = values
        .pop_front()
        .context("expected header specifying number of child nodes")?;
    let total_entries = values
        .pop_front()
        .context("expected header specifying number of metadata entries")?
        as usize;

    let mut node = Node::new();
    for _ in 0..total_children {
        let child = build_tree(values)?;
        node.children.push(child);
    }

    let mut entries = values.drain(0..total_entries).collect::<Vec<u32>>();
    if entries.len() != total_entries {
        bail!(
            "expected {} metadata entries but got {}",
            total_entries,
            entries.len()
        );
    }

    node.entries.append(&mut entries);

    Ok(node)
}
