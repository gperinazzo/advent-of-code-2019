use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn read_orbits() -> HashMap<String, String> {
    stdin()
        .lock()
        .lines()
        .map(Result::unwrap)
        .map(|line| {
            let mut split = line.split(")");
            let orbited = split.next().unwrap().to_owned();
            let orbits = split.next().unwrap().to_owned();
            (orbits, orbited)
        })
        .collect()
}

struct PathIterator<'a> {
    graph: &'a HashMap<String, String>,
    current: &'a str,
}

impl<'a> Iterator for PathIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.graph.get(self.current) {
            Some(value) => {
                self.current = value;
                Some(value.as_str())
            }
            None => None,
        }
    }
}

impl<'a> PathIterator<'a> {
    fn new(graph: &'a HashMap<String, String>, initial: &'a str) -> Self {
        Self {
            graph,
            current: initial,
        }
    }
}

fn orbit_count_checksum(input: &HashMap<String, String>) -> u32 {
    let mut orbits_map: HashMap<String, u32> = HashMap::with_capacity(input.len());
    for (key, value) in input.iter() {
        if orbits_map.contains_key(key) {
            continue;
        }
        let mut stack: Vec<&String> = Vec::new();
        let mut current_key = key;
        let mut current_value = value;
        let mut steps;
        loop {
            if current_value == "COM" {
                steps = 1;
                break;
            } else if let Some(value_steps) = orbits_map.get(current_value).clone() {
                steps = value_steps + 1;
                break;
            }
            stack.push(current_key);
            current_key = current_value;
            current_value = input.get(current_key).unwrap();
        }

        orbits_map.insert(current_key.clone(), steps);
        for key in stack.drain(..).rev() {
            steps += 1;
            orbits_map.insert(key.clone(), steps);
        }
    }

    orbits_map.values().sum::<u32>()
}

fn jumps_to_santa(input: &HashMap<String, String>) -> Option<u32> {
    let your_path: HashMap<&str, usize> = PathIterator::new(input, "YOU")
        .enumerate()
        .map(|(index, key)| (key, index))
        .collect();

    PathIterator::new(input, "SAN")
        .enumerate()
        .find_map(|(index, key)| your_path.get(key).map(|value| *value as u32 + index as u32))
}

fn main() {
    let input = read_orbits();
    println!("Puzzle 1: {}", orbit_count_checksum(&input));
    match jumps_to_santa(&input) {
        Some(value) => println!("Puzzle 2: {}", value),
        None => println!("Puzzle 2: no solution found"),
    };
}
