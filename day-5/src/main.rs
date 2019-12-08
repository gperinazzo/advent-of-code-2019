use intcode::{read_intcode_input, IntCodeMachine};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = BufReader::new(File::open("./input.txt").unwrap());
    let memory = read_intcode_input(file).expect("Invalid puzzle input");
    let mut machine = IntCodeMachine::new(memory.clone());
    let output = machine.execute(vec![1]).unwrap();
    println!("Puzzle 1 - {:?}", output);

    let mut machine = IntCodeMachine::new(memory.clone());
    let output = machine.execute(vec![5]).unwrap();
    println!("Puzzle 2 - {:?}", output);
}
