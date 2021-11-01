const INPUT: usize = 409551;

fn main() {
    part1(INPUT);

    let digits = INPUT
        .to_string()
        .as_bytes()
        .iter()
        .map(|c| c - b'0')
        .collect::<Vec<u8>>();
    part2(&digits);
}

fn part1(recipe_count: usize) {
    let mut recipes = Recipes::new();
    while recipes.scores.len() < recipe_count + 10 {
        recipes.step();
    }

    let (_, tail) = recipes.scores.split_at(recipe_count);
    let answer = tail.iter()
        .take(10)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .concat();

    println!("Part 1: Next 10 scores: {}", answer);
}

fn part2(digits: &[u8]) {
    let mut recipes = Recipes::new();
    let recipe_count;
    loop {
        if recipes.scores.ends_with(digits) {
            recipe_count = recipes.scores.len() - digits.len();
            break;
        }
        if recipes.scores[..recipes.scores.len() - 1].ends_with(digits) {
            recipe_count = recipes.scores.len() - digits.len() - 1;
            break;
        }
        recipes.step();
    }

    println!("Part 2: Recipes to the left: {}", recipe_count);
}

struct Recipes {
    scores: Vec<u8>,
    elves: Vec<usize>,
}

impl Recipes {
    fn new() -> Recipes {
        Recipes {
            scores: vec![3, 7],
            elves: vec![0, 1],
        }
    }

    fn step(&mut self) {
        let sum: u8 = self.elves.iter().map(|i| self.scores[*i]).sum();
        let mut new_scores = sum
            .to_string()
            .as_bytes()
            .iter()
            .map(|c| c - b'0')
            .collect::<Vec<u8>>();
        self.scores.append(&mut new_scores);

        for e in &mut self.elves {
            *e = (*e + self.scores[*e] as usize + 1) % self.scores.len();
        }
    }
}
