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

fn orbit_count_checksum(input: &HashMap<String, String>) -> usize {
    let mut orbits_map: HashMap<String, usize> = HashMap::with_capacity(input.len() + 1);
    orbits_map.insert("COM".to_string(), 0);
    for key in input.keys() {
        if orbits_map.contains_key(key) {
            continue;
        }
        let mut stack: Vec<&str> = vec![key];
        let mut steps = 0;
        for obj in PathIterator::new(input, key.as_str()) {
            if let Some(value) = orbits_map.get(obj) {
                steps = *value;
                break;
            }
            stack.push(obj);
        }

        for (index, key) in stack.drain(..).rev().enumerate() {
            orbits_map.insert(key.to_string(), steps + index + 1);
        }
    }

    orbits_map.values().sum()
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
