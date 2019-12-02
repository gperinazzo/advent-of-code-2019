use std::io::{stdin, BufRead};

fn calculate_final_fuel(mut fuel: i64) -> i64 {
    let mut extra = 0;
    while fuel > 0 {
        extra += fuel;
        fuel = (fuel / 3) - 2;
    }
    extra
}

fn get_modules_fuel() -> i64 {
    stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().parse::<i64>().unwrap())
        .map(|mass| (mass / 3) - 2)
        .map(calculate_final_fuel)
        .sum()
}

fn main() {
    let total_fuel = get_modules_fuel();
    println!("{}", total_fuel);
}
