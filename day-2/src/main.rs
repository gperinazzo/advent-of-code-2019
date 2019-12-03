use std::boxed::Box;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io::{stdin, BufRead};

type Error = Box<dyn std::error::Error>;

type Result<T, E = Error> = std::result::Result<T, E>;

fn read_input() -> Result<Vec<usize>> {
    let mut buffer = String::new();
    stdin().lock().read_line(&mut buffer)?;

    let parsed_values: Result<Vec<_>, _> = buffer
        .trim()
        .split(",")
        .map(|value| value.parse::<usize>())
        .collect();

    Ok(parsed_values?)
}

#[derive(Debug)]
enum IntCodeError {
    InvalidOpCode(usize),
    UnexpectedEndOfFile,
}

impl fmt::Display for IntCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntCodeError::InvalidOpCode(code) => write!(f, "Invalid Op Code found: {}", code),
            IntCodeError::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
        }
    }
}

impl std::error::Error for IntCodeError {}

enum Command {
    Add = 1,
    Multiply = 2,
    Exit = 99,
}

impl TryFrom<usize> for Command {
    type Error = IntCodeError;
    fn try_from(value: usize) -> Result<Self, IntCodeError> {
        match value {
            1 => Ok(Command::Add),
            2 => Ok(Command::Multiply),
            99 => Ok(Command::Exit),
            _ => Err(IntCodeError::InvalidOpCode(value)),
        }
    }
}

enum Operation {
    Add(usize, usize, usize),
    Multiply(usize, usize, usize),
    Stop,
}

impl Command {
    fn read_next(array: &[usize], position: usize) -> Result<usize, IntCodeError> {
        if array.len() > position + 1 {
            Ok(array[position])
        } else {
            Err(IntCodeError::UnexpectedEndOfFile)
        }
    }

    fn read_args(array: &[usize], position: usize) -> Result<(usize, usize, usize), IntCodeError> {
        if array.len() > position + 3 {
            Ok((array[position], array[position + 1], array[position + 2]))
        } else {
            Err(IntCodeError::UnexpectedEndOfFile)
        }
    }

    pub fn read_command(
        array: &[usize],
        position: usize,
    ) -> Result<(usize, Operation), IntCodeError> {
        let command: Command = Command::read_next(array, position)?.try_into()?;
        let position = position + 1;

        match command {
            Command::Exit => Ok((1, Operation::Stop)),
            Command::Add => {
                let args = Command::read_args(array, position)?;
                Ok((4, Operation::Add(args.0, args.1, args.2)))
            }
            Command::Multiply => {
                let args = Command::read_args(array, position)?;
                Ok((4, Operation::Multiply(args.0, args.1, args.2)))
            }
        }
    }
}

fn run_intcode(input: &mut [usize]) -> Result<(), IntCodeError> {
    let mut instruction_pointer: usize = 0;
    let length = input.len();
    loop {
        if instruction_pointer > length {
            return Err(IntCodeError::UnexpectedEndOfFile);
        }
        match Command::read_command(&input, instruction_pointer)? {
            (_, Operation::Stop) => return Ok(()),
            (advance, Operation::Add(x, y, result)) => {
                input[result] = input[x] + input[y];
                instruction_pointer += advance;
            }
            (advance, Operation::Multiply(x, y, result)) => {
                input[result] = input[x] * input[y];
                instruction_pointer += advance;
            }
        }
    }
}

fn main() {
    let mut input = read_input().expect("Invalid puzzle input");
    run_intcode(&mut input).unwrap();
    println!("First position: {}", input[0]);
}
