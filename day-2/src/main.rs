use intcode::{read_intcode_input, run_intcode};
use std::io::{stdin, stdout};

fn main() {
    let input = read_intcode_input(stdin().lock()).expect("Invalid puzzle input");
    let mut input_clone = input.clone();
    run_intcode(&mut input_clone, stdin().lock(), stdout().lock()).unwrap();
    println!("Puzzle 1 - First position: {}", input_clone[0]);

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut input = input.clone();
            input[1] = noun;
            input[2] = verb;
            run_intcode(&mut input, stdin().lock(), stdout().lock()).unwrap();
            if input[0] == 19690720 {
                println!("Found noun {}, verb {}", noun, verb);
                println!("Puzzle 2 - Result: {}", 100 * noun + verb);
                return;
            }
        }
    }
}
