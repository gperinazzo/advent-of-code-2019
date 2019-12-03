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

enum OpCode {
    Add = 1,
    Multiply = 2,
    Exit = 99,
}

impl OpCode {
    fn num_args(&self) -> usize {
        match self {
            OpCode::Add => 3,
            OpCode::Multiply => 3,
            OpCode::Exit => 0,
        }
    }
}

impl TryFrom<&usize> for OpCode {
    type Error = IntCodeError;
    fn try_from(value: &usize) -> Result<Self, IntCodeError> {
        match value {
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Multiply),
            99 => Ok(OpCode::Exit),
            _ => Err(IntCodeError::InvalidOpCode(*value)),
        }
    }
}

enum Command {
    Add(usize, usize, usize),
    Multiply(usize, usize, usize),
    Stop,
}

impl Command {
    fn from_op_code<'a, I>(op: &OpCode, iterator: &mut I) -> Result<Command, IntCodeError>
    where
        I: Iterator<Item = &'a usize>,
    {
        let len = op.num_args();
        let args = Command::read_args(iterator, len)?;
        let command = match op {
            OpCode::Add => Command::Add(args[0], args[1], args[2]),
            OpCode::Multiply => Command::Multiply(args[0], args[1], args[2]),
            OpCode::Exit => Command::Stop,
        };
        Ok(command)
    }

    fn read_next<'a, I>(iterator: &mut I) -> Result<&'a usize, IntCodeError>
    where
        I: Iterator<Item = &'a usize>,
    {
        iterator.next().ok_or(IntCodeError::UnexpectedEndOfFile)
    }

    fn read_args<'a, I>(iterator: &mut I, num: usize) -> Result<Vec<usize>, IntCodeError>
    where
        I: Iterator<Item = &'a usize>,
    {
        let values = iterator.take(num).cloned().collect::<Vec<usize>>();

        if values.len() < num {
            Err(IntCodeError::UnexpectedEndOfFile)
        } else {
            Ok(values)
        }
    }

    pub fn read_command<'a, I>(iterator: &mut I) -> Result<(usize, Command), IntCodeError>
    where
        I: Iterator<Item = &'a usize>,
    {
        let op_code: OpCode = Command::read_next(iterator)?.try_into()?;
        let command = Command::from_op_code(&op_code, iterator)?;
        Ok((op_code.num_args() + 1, command))
    }

    pub fn execute(&self, memory: &mut [usize]) {
        match self {
            Command::Stop => return,
            Command::Add(x, y, result) => {
                memory[*result] = memory[*x] + memory[*y];
            }
            Command::Multiply(x, y, result) => {
                memory[*result] = memory[*x] * memory[*y];
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
        let (advance, command) =
            Command::read_command(&mut input[instruction_pointer..].into_iter())?;
        if let Command::Stop = command {
            return Ok(());
        }

        command.execute(input);
        instruction_pointer += advance;
    }
}

fn main() {
    let input = read_input().expect("Invalid puzzle input");
    let mut input_clone = input.clone();
    run_intcode(&mut input_clone).unwrap();
    println!("Puzzle 1 - First position: {}", input_clone[0]);

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut input = input.clone();
            input[1] = noun;
            input[2] = verb;
            run_intcode(&mut input).unwrap();
            if input[0] == 19690720 {
                println!("Found noun {}, verb {}", noun, verb);
                println!("Puzzle 2 - Result: {}", 100 * noun + verb);
                return;
            }
        }
    }
}
