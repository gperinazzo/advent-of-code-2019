use std::boxed::Box;
use std::convert::{From, TryFrom, TryInto};
use std::fmt;
use std::io::{BufRead, Write};
use std::num::TryFromIntError;

type ParseError = Box<dyn std::error::Error>;

type Value = isize;

#[derive(Debug)]
pub enum IntCodeError {
    InvalidOpCode(Value),
    InvalidParameterMode(Value),
    InvalidAddress,
    ImmediateModeOutput,
    UnexpectedEndOfFile,
    IoError(std::io::Error),
    ParseError(std::num::ParseIntError),
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
            IntCodeError::IoError(err) => write!(f, "IO error: {}", err),
            IntCodeError::ParseError(err) => write!(f, "Parse error: {}", err),
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

impl From<std::io::Error> for IntCodeError {
    fn from(err: std::io::Error) -> Self {
        IntCodeError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for IntCodeError {
    fn from(err: std::num::ParseIntError) -> Self {
        IntCodeError::ParseError(err)
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

impl OpCode {
    fn num_args(&self) -> usize {
        match self {
            OpCode::Add(..) => 3,
            OpCode::Multiply(..) => 3,
            OpCode::Input(..) => 1,
            OpCode::Output(..) => 1,
            OpCode::JumpIfTrue(..) => 2,
            OpCode::JumpIfFalse(..) => 2,
            OpCode::LessThan(..) => 3,
            OpCode::Equals(..) => 3,
            OpCode::Exit => 0,
        }
    }
}

impl TryFrom<&Value> for OpCode {
    type Error = IntCodeError;
    fn try_from(value: &Value) -> Result<Self> {
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
            _ => Err(IntCodeError::InvalidOpCode(*value)),
        }
    }
}

struct Parameter {
    mode: ParameterMode,
    value: Value,
}

impl Parameter {
    fn get(&self, arr: &[Value]) -> Result<Value> {
        match self.mode {
            ParameterMode::Immediate => Ok(self.value),
            ParameterMode::Reference => {
                let address: usize = self.value.try_into()?;
                Ok(arr[address])
            }
        }
    }

    fn set(&self, value: Value, arr: &mut [Value]) -> Result<()> {
        if let ParameterMode::Immediate = self.mode {
            return Err(IntCodeError::ImmediateModeOutput);
        }
        let address: usize = self.value.try_into()?;
        arr[address] = value;
        Ok(())
    }
}

enum Command {
    Add(Parameter, Parameter, Parameter),
    Multiply(Parameter, Parameter, Parameter),
    Input(Parameter),
    Output(Parameter),
    JumpIfTrue(Parameter, Parameter),
    JumpIfFalse(Parameter, Parameter),
    LessThan(Parameter, Parameter, Parameter),
    Equals(Parameter, Parameter, Parameter),
    Stop,
}

impl Command {
    fn from_op_code<'a, I>(op: OpCode, iterator: &mut I) -> Result<Command>
    where
        I: Iterator<Item = &'a Value>,
    {
        let len = op.num_args();
        let args = Command::read_args(iterator, len)?;
        let command = match op {
            OpCode::Add(m1, m2, m3) => Command::Add(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
                Parameter {
                    mode: m3,
                    value: args[2],
                },
            ),
            OpCode::Multiply(m1, m2, m3) => Command::Multiply(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
                Parameter {
                    mode: m3,
                    value: args[2],
                },
            ),
            OpCode::Input(m1) => Command::Input(Parameter {
                mode: m1,
                value: args[0],
            }),
            OpCode::Output(m1) => Command::Output(Parameter {
                mode: m1,
                value: args[0],
            }),
            OpCode::JumpIfTrue(m1, m2) => Command::JumpIfTrue(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
            ),
            OpCode::JumpIfFalse(m1, m2) => Command::JumpIfFalse(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
            ),
            OpCode::LessThan(m1, m2, m3) => Command::LessThan(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
                Parameter {
                    mode: m3,
                    value: args[2],
                },
            ),
            OpCode::Equals(m1, m2, m3) => Command::Equals(
                Parameter {
                    mode: m1,
                    value: args[0],
                },
                Parameter {
                    mode: m2,
                    value: args[1],
                },
                Parameter {
                    mode: m3,
                    value: args[2],
                },
            ),
            OpCode::Exit => Command::Stop,
        };
        Ok(command)
    }

    fn read_next<'a, I>(iterator: &mut I) -> Result<&'a Value>
    where
        I: Iterator<Item = &'a Value>,
    {
        iterator.next().ok_or(IntCodeError::UnexpectedEndOfFile)
    }

    fn read_args<'a, I>(iterator: &mut I, num: usize) -> Result<Vec<Value>>
    where
        I: Iterator<Item = &'a Value>,
    {
        let values = iterator.take(num).cloned().collect::<Vec<Value>>();

        if values.len() < num {
            Err(IntCodeError::UnexpectedEndOfFile)
        } else {
            Ok(values)
        }
    }

    pub fn read_command<'a, I>(iterator: &mut I) -> Result<(usize, Command)>
    where
        I: Iterator<Item = &'a Value>,
    {
        let op_code: OpCode = Command::read_next(iterator)?.try_into()?;
        let num_args = op_code.num_args();
        let command = Command::from_op_code(op_code, iterator)?;
        Ok((num_args + 1, command))
    }

    pub fn execute<In, Out>(
        &self,
        memory: &mut [Value],
        mut input: In,
        mut output: Out,
    ) -> Result<Option<usize>>
    where
        In: BufRead,
        Out: Write,
    {
        match self {
            Command::Stop => return Ok(None),
            Command::Add(x, y, result) => {
                result.set(x.get(memory)? + y.get(memory)?, memory)?;
            }
            Command::Multiply(x, y, result) => {
                result.set(x.get(memory)? * y.get(memory)?, memory)?;
            }
            Command::Input(address) => {
                let mut line = String::new();
                input.read_line(&mut line)?;
                let value: Value = line.trim().parse()?;
                address.set(value, memory)?;
            }
            Command::Output(address) => {
                write!(output, "{}\n", address.get(memory)?)?;
            }
            Command::LessThan(x, y, result) => {
                if x.get(memory)? < y.get(memory)? {
                    result.set(1, memory)?;
                } else {
                    result.set(0, memory)?;
                }
            }
            Command::Equals(x, y, result) => {
                if x.get(memory)? == y.get(memory)? {
                    result.set(1, memory)?;
                } else {
                    result.set(0, memory)?;
                }
            }
            Command::JumpIfTrue(cond, address) => {
                if cond.get(memory)? != 0 {
                    return Ok(Some(address.get(memory)?.try_into()?));
                }
            }
            Command::JumpIfFalse(cond, address) => {
                if cond.get(memory)? == 0 {
                    return Ok(Some(address.get(memory)?.try_into()?));
                }
            }
        }
        Ok(None)
    }
}

pub fn run_intcode<In, Out>(memory: &mut [Value], mut input: In, mut output: Out) -> Result<Out>
where
    In: BufRead,
    Out: Write,
{
    let mut instruction_pointer: usize = 0;
    let length = memory.len();
    loop {
        if instruction_pointer > length {
            return Err(IntCodeError::UnexpectedEndOfFile);
        }
        let (advance, command) =
            Command::read_command(&mut memory[instruction_pointer..].into_iter())?;
        if let Command::Stop = command {
            return Ok(output);
        }

        if let Some(jump) = command.execute(memory, &mut input, &mut output)? {
            instruction_pointer = jump;
        } else {
            instruction_pointer += advance;
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
        .split(",")
        .map(|value| value.parse::<Value>())
        .collect();
    Ok(parsed_values?)
}

#[cfg(test)]
mod test {
    use super::run_intcode;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_case_1() {
        let buffer: &[u8] = &[];
        let mut input = vec![1, 0, 0, 0, 99];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [2, 0, 0, 0, 99]);
    }

    #[test]
    fn test_case_2() {
        let buffer: &[u8] = &[];
        let mut input = vec![2, 3, 0, 3, 99];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [2, 3, 0, 6, 99]);
    }

    #[test]
    fn test_case_3() {
        let buffer: &[u8] = &[];
        let mut input = vec![2, 4, 4, 5, 99, 0];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [2, 4, 4, 5, 99, 9801]);
    }

    #[test]
    fn test_case_4() {
        let buffer: &[u8] = &[];
        let mut input = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_case_5() {
        let buffer: &[u8] = &[];
        let mut input = vec![1002, 4, 3, 4, 33];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [1002, 4, 3, 4, 99]);
    }
    #[test]
    fn test_case_6() {
        let buffer: &[u8] = b"99";
        let mut input = vec![3, 2, 0];
        run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(input[..], [3, 2, 99]);
    }
    #[test]
    fn test_case_7() {
        let buffer: &[u8] = b"0";
        let mut input = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let output = run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(&output.into_inner()[..], "0\n".as_bytes())
    }

    #[test]
    fn test_case_8() {
        let buffer: &[u8] = b"0";
        let mut input = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let output = run_intcode(&mut input, BufReader::new(buffer), Cursor::new(vec![]))
            .expect("Expect to work");
        assert_eq!(&output.into_inner()[..], "0\n".as_bytes())
    }
}
