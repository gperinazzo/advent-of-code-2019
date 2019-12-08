use intcode::{read_intcode_input, IntCodeError, IntCodeMachine, Machine, Pipe};
use itertools::Itertools;
use std::boxed::Box;
use std::cmp;
use std::collections::HashSet;
use std::io::stdin;

fn run_to_completion(mut machine: Box<dyn Machine>) -> Result<Vec<isize>, IntCodeError> {
    let mut out = machine.execute(vec![0])?;
    while !machine.finished() {
        out = machine.execute(out)?;
    }
    Ok(out)
}

fn max_signal(memory: &[isize], phases: HashSet<isize>) -> Result<isize, IntCodeError> {
    let mut max: Option<isize> = None;
    for permutation in phases.iter().permutations(5) {
        let mut machines = permutation.into_iter().map(|start| {
            let mut machine: Box<dyn Machine> = Box::new(IntCodeMachine::new(memory.to_vec()));
            machine.execute(vec![*start]).unwrap();
            machine
        });
        let mut machine = machines.next().unwrap();
        for next_machine in machines {
            machine = Box::new(Pipe::new(machine, next_machine));
        }

        let value = run_to_completion(machine)?;
        max = Some(max.map_or(value[0], |prev_max| cmp::max(prev_max, value[0])))
    }
    Ok(max.unwrap())
}

fn main() {
    let input = read_intcode_input(stdin().lock()).unwrap();
    let phases = [0, 1, 2, 3, 4].iter().cloned().collect();
    println!("Puzzle 1 - {}", max_signal(&input, phases).unwrap());

    let phases = [5, 6, 7, 8, 9].iter().cloned().collect();
    println!("Puzzle 2 - {}", max_signal(&input, phases).unwrap());
}

#[cfg(test)]
mod test {
    use super::max_signal;

    #[test]
    fn test_case_1() {
        let program = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        let phases = [0, 1, 2, 3, 4].iter().cloned().collect();
        assert_eq!(max_signal(&program, phases).unwrap(), 43210);
    }

    #[test]
    fn test_case_2() {
        let program = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        let phases = [0, 1, 2, 3, 4].iter().cloned().collect();
        assert_eq!(max_signal(&program, phases).unwrap(), 54321);
    }

    #[test]
    fn test_case_3() {
        let program = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        let phases = [0, 1, 2, 3, 4].iter().cloned().collect();
        assert_eq!(max_signal(&program, phases).unwrap(), 65210);
    }
}
