use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
};

#[derive(Copy, Clone)]
enum Mode {
    Position(usize),
    Immediate(i32),
}

impl Mode {
    fn from(op_code: usize, mem: &[i32], idx: usize, off: usize) -> Self {
        let mode = op_code / 10usize.pow(off as u32) % 10;
        let val = mem[idx + off];
        if mode == 0 {
            Mode::Position(val as usize)
        } else {
            Mode::Immediate(val)
        }
    }

    fn val(&self, mem: &[i32]) -> i32 {
        match *self {
            Mode::Position(idx) => {
                if idx >= mem.len() {
                    0
                } else {
                    mem[idx]
                }
            }
            Mode::Immediate(val) => val,
        }
    }
}

trait Io {
    fn read(&self) -> String;
    fn write(&self, output: &str);
}

struct RealIo;

impl Io for RealIo {
    fn read(&self) -> String {
        print!("Input: ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        handle
            .read_line(&mut buffer)
            .expect("failed to read from stdin");
        buffer
    }

    fn write(&self, output: &str) {
        println!("{}", output);
    }
}

enum Incr {
    Offset(usize),
    Jump(usize),
    Exit,
}

#[derive(Copy, Clone)]
// Models the possible commands available to this "machine"
enum OpCode {
    Add { r1: Mode, r2: Mode, o: usize },
    Mul { r1: Mode, r2: Mode, o: usize },
    Ipt { adr: usize },
    Opt { o: Mode },
    Jtr { r1: Mode, jmp: Mode },
    Jfl { r1: Mode, jmp: Mode },
    Les { r1: Mode, r2: Mode, o: usize },
    Eql { r1: Mode, r2: Mode, o: usize },
    Ext,
}

impl OpCode {
    /// Generate an OpCode from a specific region of memory
    fn from(mem: &[i32], idx: usize) -> Self {
        let instruction = mem[idx];
        let op_code = instruction % 100;
        let mode_spec = instruction as usize / 100;
        let pidx = idx + 1;
        match op_code {
            1 => OpCode::Add {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: mem[pidx + 2] as usize,
            },
            2 => OpCode::Mul {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: mem[pidx + 2] as usize,
            },
            3 => OpCode::Ipt {
                adr: mem[pidx] as usize,
            },
            4 => OpCode::Opt {
                o: Mode::from(mode_spec, mem, pidx, 0),
            },
            5 => OpCode::Jtr {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                jmp: Mode::from(mode_spec, mem, pidx, 1),
            },
            6 => OpCode::Jfl {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                jmp: Mode::from(mode_spec, mem, pidx, 1),
            },
            7 => OpCode::Les {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: mem[pidx + 2] as usize,
            },
            8 => OpCode::Eql {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: mem[pidx + 2] as usize,
            },
            99 => OpCode::Ext,
            _ => panic!(format!("Unexpected opcode: {}", op_code)),
        }
    }

    // Execute the OpCode against the passed in memory
    fn exec(self, mem: &mut [i32], io: &impl Io) -> Incr {
        match self {
            OpCode::Ext => return Incr::Exit,
            OpCode::Add { r1, r2, o } => mem[o] = r1.val(mem) + r2.val(mem),
            OpCode::Mul { r1, r2, o } => mem[o] = r1.val(mem) * r2.val(mem),
            OpCode::Ipt { adr } => {
                mem[adr] = io.read().trim().parse().expect("Failed to parse input")
            }
            OpCode::Opt { o } => io.write(&format!("{}", o.val(mem))),
            OpCode::Jtr { r1, jmp } => {
                if r1.val(mem) != 0 {
                    return Incr::Jump(jmp.val(mem) as usize);
                }
            }
            OpCode::Jfl { r1, jmp } => {
                if r1.val(mem) == 0 {
                    return Incr::Jump(jmp.val(mem) as usize);
                }
            }
            OpCode::Les { r1, r2, o } => mem[o] = if r1.val(mem) < r2.val(mem) { 1 } else { 0 },
            OpCode::Eql { r1, r2, o } => mem[o] = if r1.val(mem) == r2.val(mem) { 1 } else { 0 },
        }

        Incr::Offset(self.len())
    }

    fn len(&self) -> usize {
        match self {
            OpCode::Add { .. } | OpCode::Mul { .. } | OpCode::Les { .. } | OpCode::Eql { .. } => 4,
            OpCode::Jtr { .. } | OpCode::Jfl { .. } => 3,
            OpCode::Ipt { .. } | OpCode::Opt { .. } => 2,
            OpCode::Ext => 1,
        }
    }
}

// Models a simple machine with memory and a program counter
struct Program<'a, T>
where
    T: Io,
{
    mem: &'a mut [i32],
    ctr: usize,
    io: &'a T,
}

impl<'a, T> Program<'a, T>
where
    T: Io,
{
    fn new(mem: &'a mut [i32], io: &'a T) -> Self {
        Program { mem, ctr: 0, io }
    }

    // Process the op codes in memory until an exit opcode is reached
    fn run(mut self) {
        loop {
            let op_code = OpCode::from(self.mem, self.ctr);
            self.ctr = match op_code.exec(self.mem, self.io) {
                Incr::Offset(offset) => self.ctr + offset,
                Incr::Jump(address) => address,
                Incr::Exit => break,
            }
        }
    }
}

fn main() -> Result<(), String> {
    let mut mem = parse("day5/input.txt")?;
    let program = Program::new(&mut mem, &RealIo);
    program.run();
    println!("Total memory is: {:?}", mem);
    Ok(())
}

fn parse(file_name: &str) -> Result<Vec<i32>, String> {
    let mut file = File::open(file_name).map_err(|e| format!("Failed to open input: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(contents
        .split(",")
        .filter_map(|s| {
            let s = s.trim();
            if s.len() == 0 {
                None
            } else {
                s.parse::<i32>().ok()
            }
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{cell::RefCell, collections::VecDeque};

    #[cfg(test)]
    struct MockIo {
        input: RefCell<VecDeque<String>>,
        output: RefCell<Vec<String>>,
    }

    impl MockIo {
        fn new() -> Self {
            Self {
                input: RefCell::new(VecDeque::new()),
                output: RefCell::new(vec![]),
            }
        }

        fn with_input(input: &[&str]) -> Self {
            Self {
                input: RefCell::new(input.iter().map(|&s| s.to_owned()).collect()),
                output: RefCell::new(vec![]),
            }
        }
    }

    impl Io for MockIo {
        fn read(&self) -> String {
            self.input
                .borrow_mut()
                .pop_front()
                .expect("Ran out of input")
        }

        fn write(&self, output: &str) {
            self.output.borrow_mut().push(output.to_owned());
        }
    }

    macro_rules! validate_program {
        ($mem:expr, $expected:expr) => {
            let io = MockIo::new();
            let mem = $mem;
            let program = Program::new(mem, &io);
            let expected = $expected;
            program.run();
            assert_eq!(
                mem.len(),
                expected.len(),
                "mem and expected mem are not the same length:\n{:?}\n{:?}",
                &mem[..],
                &expected[..]
            );
            for (i, (l, r)) in mem.iter().zip(expected.iter()).enumerate() {
                assert_eq!(l, r, "mem mismatch at idx {}", i);
            }
        };
    }

    #[test]
    fn test_run() {
        // Add value at address 0 to itself and store
        // it in address 0
        #[rustfmt::skip]
        validate_program!(
            &mut [1, 0, 0, 0, 99],
                &[2, 0, 0, 0, 99]);

        // Multiply value at address 3 with value at
        // address 0 and store it in address 3
        #[rustfmt::skip]
        validate_program!(
            &mut [2, 3, 0, 3, 99],
                &[2, 3, 0, 6, 99]);

        // Multiply value at address 4 with value itself
        // and store it in address 5
        #[rustfmt::skip]
        validate_program!(
            &mut [2, 4, 4, 5, 99, 0],
                &[2, 4, 4, 5, 99, 9801]);

        // Add value at address 1 to itself and store it
        // in address 4 (create mul opcode)
        // Multiply value at address 5 with value at
        // address 6 and store it in address 0
        #[rustfmt::skip]
        validate_program!(
            &mut [ 1, 1, 1, 4, 99, 5, 6, 0, 99],
                &[30, 1, 1, 4,  2, 5, 6, 0, 99]);
    }

    macro_rules! validate_program_with_io {
        ($mem:expr, $expected:expr, $input:expr, $output:expr $(,)?) => {
            let io = MockIo::with_input($input);
            let mem = $mem;
            let expected = $expected;
            let program = Program::new(mem, &io);
            program.run();
            assert_eq!(
                mem.len(),
                expected.len(),
                "mem and expected mem are not the same length:\n{:?}\n{:?}",
                &mem[..],
                &expected[..]
            );
            for (i, (l, r)) in mem.iter().zip(expected.iter()).enumerate() {
                assert_eq!(l, r, "mem mismatch at idx {}", i);
            }
            let output: &[&str] = $output;
            assert_eq!(
                io.output.borrow().len(),
                output.len(),
                "output and expected output are not the same length:\n{:?}\n{:?}",
                &io.output.borrow()[..],
                &output[..]
            );
            for (i, (l, r)) in io.output.borrow().iter().zip(output.iter()).enumerate() {
                assert_eq!(l, r, "output mismatch at idx {}", i);
            }
        };
    }

    // Position mode tests
    #[test]
    fn position_op_equal_marks_true_on_equal() {
        // Read input of "8" and store it in address 9
        // Since value at address 9 is equal to value at
        // address 10 (8 == 8), store 1 in address 9
        // Output value at address 9 (1)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
                &[3, 9, 8, 9, 10, 9, 4, 9, 99,  1, 8],
            &["8\n"],
            &["1"],
        );
    }

    #[test]
    fn position_op_equal_marks_false_on_not_equal() {
        // Read input of "7" and store it in address 9
        // Since value at address 9 is not equal to value
        // at address 10 (7 != 8), store 0 in address 9
        // Output value at address 9 (0)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
                &[3, 9, 8, 9, 10, 9, 4, 9, 99,  0, 8],
            &["7\n"],
            &["0"],
        );
    }

    #[test]
    fn position_op_less_marks_true_on_less() {
        // Read input of "7" and store it in address 9
        // Since value at address 9 is less than the
        // value at address 10 (7 < 8), store 1 in
        // address 9. Output value at address 9 (1)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
                &[3, 9, 7, 9, 10, 9, 4, 9, 99,  1, 8],
            &["7\n"],
            &["1"],
        );
    }

    #[test]
    fn position_op_less_marks_false_on_not_less() {
        // Read input of "8" and store it in address 9
        // Since value at address 9 is not less than
        // the value at address 10 (8 !< 8), store 0
        // in address 9. Output value at address 9 (0)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
                &[3, 9, 7, 9, 10, 9, 4, 9, 99,  0, 8],
            &["8\n"],
            &["0"],
        );
    }

    #[test]
    fn position_op_jtr_executes_jump_if_val_true() {
        // Read "1" from input and store it in address 12
        // Jump to the value at address 15 (address 9) because the value
        // at address 12 is not equal to 0 (1). Print the value at
        // address 14 (1).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
                &[3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99,  1, 0, 1, 9],
            &["1\n"],
            &["1"],
        );

        // Same as above, but ensure we don't only consider 1 to be "true".
        // Read "-1" from input and store it in address 12
        // Jump to the value at address 15 (address 9) because the value
        // at address 12 is not equal to 0 (-1). Print the value at
        // address 14 (1).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
                &[3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
            &["-1\n"],
            &["1"],
        );
    }

    #[test]
    fn position_op_jtr_does_not_execute_jump_if_val_false() {
        // Read input "0" into address 12
        // Do not jump to the address specified at address 15 (9)
        // because address 12 is 0. Output the value in address
        // 13 (0).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
                &[3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99,  0, 0, 0, 9],
            &["0\n"],
            &["0"],
        );
    }

    #[test]
    fn position_op_jfl_executes_jump_if_val_false() {
        // Read input "0" into address 12. Jump to the address specified
        // at address 15 (9) because address 12 is 0. Output the value in
        // address 13 (0).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
                &[3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99,  0, 0, 1, 9],
            &["0\n"],
            &["0"],
        );
    }

    #[test]
    fn position_op_jfl_does_not_execute_jump_if_val_true() {
        // Read input "1" into address 12. Do not jump to the address
        // specified at address 15 (9) because address 12 is not 0.
        // Add the values at address 13 and 14 and store them at address
        // 13 (0 + 1 => 1). Output the value in address 13 (1).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
                &[3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99,  1, 1, 1, 9],
            &["1\n"],
            &["1"],
        );
    }

    // immediate mode tests

    #[test]
    fn immediate_op_equal_marks_true_on_equal() {
        // Read input of "8" and store it in address 3
        // Since the first param is equal to the second
        // param (8 == 8), store 1 in address 3
        // Output value at address 3 (1)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1108, -1, 8, 3, 4, 3, 99],
                &[3, 3, 1108,  1, 8, 3, 4, 3, 99],
            &["8\n"],
            &["1"],
        );
    }

    #[test]
    fn immediate_op_equal_marks_false_on_not_equal() {
        // Read input of "7" and store it in address 3
        // Since the first param is equal to the second
        // param (7 != 8), store 0 in address 3
        // Output value at address 3 (0)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1108, -1, 8, 3, 4, 3, 99],
                &[3, 3, 1108,  0, 8, 3, 4, 3, 99],
            &["7\n"],
            &["0"],
        );
    }

    #[test]
    fn immediate_op_less_marks_true_on_less() {
        // Read input of "7" and store it in address 3
        // Since the first param is less than the
        // second param (7 < 8), store 1 in address 3
        // Output value at address 3 (1)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1107, -1, 8, 3, 4, 3, 99],
                &[3, 3, 1107,  1, 8, 3, 4, 3, 99],
            &["7\n"],
            &["1"],
        );
    }

    #[test]
    fn immediate_op_less_marks_false_on_not_less() {
        // Read input of "8" and store it in address 3
        // Since the first param is not less than the
        // second param (8 !< 8), store 0 in address 3
        // Output value at address 3 (0)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1107, -1, 8, 3, 4, 3, 99],
                &[3, 3, 1107,  0, 8, 3, 4, 3, 99],
            &["8\n"],
            &["0"],
        );
    }

    #[test]
    fn immediate_op_jtr_executes_jump_if_val_true() {
        // Read input of "1" and store it in address 3
        // Jump to address 9 because the param is 1
        // Output the value at address 12 (1)
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
                &[3, 3, 1105,  1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
            &["1\n"],
            &["1"],
        );
    }

    #[test]
    fn immediate_op_jtr_does_not_execute_jump_if_val_false() {
        // Read input "0" into address 3. Do not jump to address 9
        // because the first param is 0. Add 0 with itself and
        // store it in address 12. Output the value in address 12
        // (0).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
                &[3, 3, 1105,  0, 9, 1101, 0, 0, 12, 4, 12, 99, 0],
            &["0\n"],
            &["0"],
        );
    }

    #[test]
    fn immediate_op_jfl_executes_jump_if_val_false() {
        // Read input "0" into address 3. Jump to address 9 because
        // the first param is 0. Output the value in address 12 (0).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1106, -1, 9, 1101, 0, 1, 12, 4, 12, 99, 0],
                &[3, 3, 1106,  0, 9, 1101, 0, 1, 12, 4, 12, 99, 0],
            &["0\n"],
            &["0"],
        );
    }

    #[test]
    fn immediate_op_jfl_does_not_execute_jump_if_val_true() {
        // Read input "1" into address 3. Do not jump to address 9
        // because the first param is not 0. Add 0 to 1 and store
        // the result in address 12 (1). Output the value in
        // address 12 (1).
        #[rustfmt::skip]
        validate_program_with_io!(
            &mut [3, 3, 1106, -1, 9, 1101, 0, 1, 12, 4, 12, 99, 0],
                &[3, 3, 1106,  1, 9, 1101, 0, 1, 12, 4, 12, 99, 1],
            &["1\n"],
            &["1"],
        );
    }

    #[test]
    fn big_test_lower_prints_999() {
        validate_program_with_io!(
            &mut [
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &[
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 7, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &["7\n"],
            &["999"],
        );
    }

    #[test]
    fn big_test_lower_prints_1000() {
        validate_program_with_io!(
            &mut [
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &[
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 1000, 8, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101,
                1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &["8\n"],
            &["1000"],
        );
    }

    #[test]
    fn big_test_lower_prints_1001() {
        validate_program_with_io!(
            &mut [
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &[
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 1001, 9, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101,
                1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            &["9\n"],
            &["1001"],
        );
    }
}
