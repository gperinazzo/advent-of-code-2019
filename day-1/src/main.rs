use std::io::{stdin, BufRead};

fn main() {
    let sum: u64 = stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<u64>().unwrap())
        .map(|mass| (mass / 3) - 2)
        .sum();
    println!("{}", sum);
}
