use intcode::{read_intcode_input, run_intcode};
use std::fs::File;
use std::io::{stdin, stdout, BufReader};

fn main() {
    let file = BufReader::new(File::open("./input.txt").unwrap());
    let mut input = read_intcode_input(file).expect("Invalid puzzle input");
    run_intcode(&mut input, stdin().lock(), stdout().lock()).unwrap();
}
