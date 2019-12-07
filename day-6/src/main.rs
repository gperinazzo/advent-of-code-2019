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

fn main() {
    let input = read_orbits();
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

    println!("Puzzle 1- {}", orbits_map.values().sum::<u32>());
}
