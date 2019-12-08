use intcode::{read_intcode_input, IntCodeMachine};
use std::io::stdin;

fn main() {
    let input = read_intcode_input(stdin().lock()).expect("Invalid puzzle input");
    let input_clone = input.clone();
    let mut machine = IntCodeMachine::new(input_clone);
    machine.execute(vec![]).unwrap();
    println!("Puzzle 1 - First position: {}", machine.memory()[0]);

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut input = input.clone();
            input[1] = noun;
            input[2] = verb;
            let mut machine = IntCodeMachine::new(input);
            machine.execute(vec![]).unwrap();
            if machine.memory()[0] == 19_690_720 {
                println!("Found noun {}, verb {}", noun, verb);
                println!("Puzzle 2 - Result: {}", 100 * noun + verb);
                return;
            }
        }
    }
}
