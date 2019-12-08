use std::boxed::Box;
use std::convert::{From, TryFrom, TryInto};
use std::fmt;
use std::io::BufRead;
use std::num::TryFromIntError;

type ParseError = Box<dyn std::error::Error>;

type Value = isize;

pub struct Pipe<M1, M2> {
    first: M1,
    second: M2,
}

impl<M1: Clone, M2: Clone> Clone for Pipe<M1, M2> {
    fn clone(&self) -> Self {
        Self {
            first: self.first.clone(),
            second: self.second.clone(),
        }
    }
}

pub trait Machine {
    fn execute(&mut self, input: Vec<Value>) -> Result<Vec<Value>>;
    fn finished(&self) -> bool;

    fn pipe<T: Machine>(self, other: T) -> Pipe<Self, T>
    where
        Self: Sized,
    {
        Pipe {
            first: self,
            second: other,
        }
    }
}

impl<T1, T2> Pipe<T1, T2> {
    pub fn new(first: T1, second: T2) -> Self {
        Self { first, second }
    }
}

impl<M1, M2> Machine for Pipe<M1, M2>
where
    M1: Machine,
    M2: Machine,
{
    fn execute(&mut self, input: Vec<Value>) -> Result<Vec<Value>> {
        let out = self.first.execute(input)?;
        self.second.execute(out)
    }

    fn finished(&self) -> bool {
        self.first.finished() || self.second.finished()
    }
}

impl Machine for Pipe<Box<dyn Machine>, Box<dyn Machine>> {
    fn execute(&mut self, input: Vec<Value>) -> Result<Vec<Value>> {
        let out = self.first.execute(input)?;
        self.second.execute(out)
    }

    fn finished(&self) -> bool {
        self.first.finished() || self.second.finished()
    }
}

#[derive(Debug)]
pub enum IntCodeError {
    InvalidOpCode(Value),
    InvalidParameterMode(Value),
    InvalidAddress,
    ImmediateModeOutput,
    UnexpectedEndOfFile,
    InputError,
}

type Result<T, E = IntCodeError> = std::result::Result<T, E>;

impl fmt::Display for IntCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntCodeError::InvalidOpCode(code) => write!(f, "Invalid Op Code found: {}", code),
            IntCodeError::InvalidParameterMode(code) => {
                write!(f, "Invalid Parameter Mode found: {}", code)
            }
            IntCodeError::InvalidAddress => write!(f, "Found invalid address"),
            IntCodeError::InputError => write!(f, "Expected input, found none"),
            IntCodeError::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
            IntCodeError::ImmediateModeOutput => {
                write!(f, "Instruction was set to output in immediate mode")
            }
        }
    }
}

impl From<TryFromIntError> for IntCodeError {
    fn from(_: TryFromIntError) -> Self {
        IntCodeError::InvalidAddress
    }
}

impl std::error::Error for IntCodeError {}

enum ParameterMode {
    Reference,
    Immediate,
}

enum OpCode {
    Add(ParameterMode, ParameterMode, ParameterMode),
    Multiply(ParameterMode, ParameterMode, ParameterMode),
    Input(ParameterMode),
    Output(ParameterMode),
    JumpIfTrue(ParameterMode, ParameterMode),
    JumpIfFalse(ParameterMode, ParameterMode),
    LessThan(ParameterMode, ParameterMode, ParameterMode),
    Equals(ParameterMode, ParameterMode, ParameterMode),
    Exit,
}

impl TryFrom<Value> for ParameterMode {
    type Error = IntCodeError;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            0 => Ok(ParameterMode::Reference),
            1 => Ok(ParameterMode::Immediate),
            _ => Err(IntCodeError::InvalidParameterMode(value)),
        }
    }
}

impl TryFrom<Value> for OpCode {
    type Error = IntCodeError;
    fn try_from(value: Value) -> Result<Self> {
        let modes = value / 100;
        macro_rules! params {
            (1) => {
                (modes % 10).try_into()?
            };
            (2) => {
                ((modes / 10) % 10).try_into()?
            };
            (3) => {
                ((modes / 100) % 10).try_into()?
            };
        }
        match value % 100 {
            1 => Ok(OpCode::Add(params!(1), params!(2), params!(3))),
            2 => Ok(OpCode::Multiply(params!(1), params!(2), params!(3))),
            3 => Ok(OpCode::Input(params!(1))),
            4 => Ok(OpCode::Output(params!(1))),
            5 => Ok(OpCode::JumpIfTrue(params!(1), params!(2))),
            6 => Ok(OpCode::JumpIfFalse(params!(1), params!(2))),
            7 => Ok(OpCode::LessThan(params!(1), params!(2), params!(3))),
            8 => Ok(OpCode::Equals(params!(1), params!(2), params!(3))),
            99 => Ok(OpCode::Exit),
            _ => Err(IntCodeError::InvalidOpCode(value)),
        }
    }
}

#[derive(Clone)]
pub enum IntCodeMachineState {
    InputRequired,
    Running,
    Finished,
}

#[derive(Clone)]
pub struct IntCodeMachine {
    memory: Vec<Value>,
    instruction_pointer: usize,
    state: IntCodeMachineState,
}

impl IntCodeMachine {
    pub fn new(memory: Vec<Value>) -> Self {
        Self {
            memory,
            instruction_pointer: 0,
            state: IntCodeMachineState::InputRequired,
        }
    }

    fn read_op_code(&mut self) -> Result<OpCode> {
        let op_code = self.memory[self.instruction_pointer].try_into()?;
        self.instruction_pointer += 1;
        Ok(op_code)
    }

    fn read_parameter(&mut self, mode: ParameterMode) -> Result<Value> {
        let current = self.memory[self.instruction_pointer];
        self.instruction_pointer += 1;
        Ok(match mode {
            ParameterMode::Immediate => current,
            ParameterMode::Reference => {
                let addr: usize = current.try_into()?;
                self.memory[addr]
            }
        })
    }

    fn read_address(&mut self, mode: ParameterMode) -> Result<usize> {
        let current = self.memory[self.instruction_pointer];
        self.instruction_pointer += 1;
        match mode {
            ParameterMode::Reference => Ok(current.try_into()?),
            ParameterMode::Immediate => Err(IntCodeError::ImmediateModeOutput),
        }
    }

    fn execute_command(
        &mut self,
        code: OpCode,
        input: &mut Vec<Value>,
        output: &mut Vec<Value>,
    ) -> Result<()> {
        match code {
            OpCode::Exit => {
                self.state = IntCodeMachineState::Finished;
            }
            OpCode::Add(m1, m2, m3) => {
                let x = self.read_parameter(m1)?;
                let y = self.read_parameter(m2)?;
                let addr = self.read_address(m3)?;
                self.memory[addr] = x + y;
            }
            OpCode::Multiply(m1, m2, m3) => {
                let x = self.read_parameter(m1)?;
                let y = self.read_parameter(m2)?;
                let addr = self.read_address(m3)?;
                self.memory[addr] = x * y;
            }
            OpCode::Input(mode) => {
                if input.is_empty() {
                    self.state = IntCodeMachineState::InputRequired;
                    self.instruction_pointer -= 1;
                } else {
                    let value = input.remove(0);
                    let addr = self.read_address(mode)?;
                    self.memory[addr] = value;
                }
            }
            OpCode::Output(mode) => {
                let value = self.read_parameter(mode)?;
                output.push(value);
            }
            OpCode::LessThan(m1, m2, m3) => {
                let x = self.read_parameter(m1)?;
                let y = self.read_parameter(m2)?;
                let addr = self.read_address(m3)?;
                if x < y {
                    self.memory[addr] = 1;
                } else {
                    self.memory[addr] = 0;
                }
            }
            OpCode::Equals(m1, m2, m3) => {
                let x = self.read_parameter(m1)?;
                let y = self.read_parameter(m2)?;
                let addr = self.read_address(m3)?;
                if x == y {
                    self.memory[addr] = 1;
                } else {
                    self.memory[addr] = 0;
                }
            }
            OpCode::JumpIfTrue(m1, m2) => {
                let cond = self.read_parameter(m1)?;
                let addr = self.read_parameter(m2)?;
                if cond != 0 {
                    self.instruction_pointer = addr.try_into()?;
                }
            }
            OpCode::JumpIfFalse(m1, m2) => {
                let cond = self.read_parameter(m1)?;
                let addr = self.read_parameter(m2)?;
                if cond == 0 {
                    self.instruction_pointer = addr.try_into()?;
                }
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, mut input: Vec<Value>) -> Result<Vec<Value>> {
        let mut output = Vec::new();
        let length = self.memory.len();
        self.state = IntCodeMachineState::Running;
        while let IntCodeMachineState::Running = self.state {
            if self.instruction_pointer > length {
                return Err(IntCodeError::UnexpectedEndOfFile);
            }
            let code = self.read_op_code()?;
            self.execute_command(code, &mut input, &mut output)?;
        }
        Ok(output)
    }

    pub fn memory(&self) -> &[Value] {
        &self.memory
    }
}

impl Machine for IntCodeMachine {
    fn execute(&mut self, mut input: Vec<Value>) -> Result<Vec<Value>> {
        let mut output = Vec::new();
        let length = self.memory.len();
        self.state = IntCodeMachineState::Running;
        while let IntCodeMachineState::Running = self.state {
            if self.instruction_pointer > length {
                return Err(IntCodeError::UnexpectedEndOfFile);
            }
            let code = self.read_op_code()?;
            self.execute_command(code, &mut input, &mut output)?;
        }
        Ok(output)
    }

    fn finished(&self) -> bool {
        match self.state {
            IntCodeMachineState::Finished => true,
            _ => false,
        }
    }
}

pub fn read_intcode_input<T>(mut input: T) -> Result<Vec<Value>, ParseError>
where
    T: BufRead,
{
    let mut buffer = String::new();
    input.read_line(&mut buffer)?;

    let parsed_values: Result<Vec<_>, _> = buffer
        .trim()
        .split(',')
        .map(|value| value.parse::<Value>())
        .collect();
    Ok(parsed_values?)
}

#[cfg(test)]
mod test {
    use super::IntCodeMachine;

    #[test]
    fn test_case_1() {
        let mut machine = IntCodeMachine::new(vec![1, 0, 0, 0, 99]);
        machine.execute(vec![]).expect("Expect to work");
        assert_eq!(machine.memory(), [2, 0, 0, 0, 99]);
    }

    #[test]
    fn test_case_2() {
        let mut machine = IntCodeMachine::new(vec![2, 3, 0, 3, 99]);
        machine.execute(vec![]).expect("Expect to work");
        assert_eq!(machine.memory(), [2, 3, 0, 6, 99]);
    }

    #[test]
    fn test_case_3() {
        let mut machine = IntCodeMachine::new(vec![2, 4, 4, 5, 99, 0]);
        machine.execute(vec![]).expect("Expect to work");
        assert_eq!(machine.memory(), [2, 4, 4, 5, 99, 9801]);
    }

    #[test]
    fn test_case_4() {
        let mut machine = IntCodeMachine::new(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        machine.execute(vec![]).expect("Expect to work");
        assert_eq!(machine.memory(), [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_case_5() {
        let mut machine = IntCodeMachine::new(vec![1002, 4, 3, 4, 33]);
        machine.execute(vec![]).expect("Expect to work");
        assert_eq!(machine.memory(), [1002, 4, 3, 4, 99]);
    }
    #[test]
    fn test_case_6() {
        let mut machine = IntCodeMachine::new(vec![3, 2, 0]);
        machine.execute(vec![99]).expect("Expect to work");
        assert_eq!(machine.memory(), [3, 2, 99]);
    }
    #[test]
    fn test_case_7() {
        let mut machine = IntCodeMachine::new(vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);
        let output = machine.execute(vec![0]).expect("Expect to work");
        assert_eq!(output, [0]);
    }

    #[test]
    fn test_case_8() {
        let mut machine =
            IntCodeMachine::new(vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);
        let output = machine.execute(vec![0]).expect("Expect to work");
        assert_eq!(output, [0]);
    }
}
