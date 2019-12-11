use std::{
    cell::RefCell,
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, Read, Write},
    // sync::mpsc::{self, Receiver, Sender},
};

#[derive(Copy, Clone, Debug)]
enum Mode {
    Position(usize),
    Immediate(i128),
    Relative(isize),
}

impl Mode {
    fn from(op_code: usize, mem: &[i128], idx: usize, off: usize) -> Self {
        let mode = op_code / 10usize.pow(off as u32) % 10;
        let val = mem[idx + off];
        match mode {
            0 => Mode::Position(val as usize),
            1 => Mode::Immediate(val),
            2 => Mode::Relative(val as isize),
            _ => panic!("Unexpected mode param {}", mode),
        }
    }

    fn val(&self, mem: &[i128], relative: usize) -> i128 {
        match *self {
            Mode::Position(idx) => {
                if idx >= mem.len() {
                    0
                } else {
                    mem[idx]
                }
            }
            Mode::Immediate(val) => val,
            Mode::Relative(offset) => {
                let idx = (relative as isize + offset) as usize;
                if idx >= mem.len() {
                    0
                } else {
                    mem[idx]
                }
            }
        }
    }

    fn adr(&self, relative: usize) -> usize {
        match *self {
            Mode::Position(adr) => adr,
            Mode::Relative(off) => {
                (relative as isize + off) as usize
            },
            _ => panic!("Addresses in immediate mode not supported"),
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

#[derive(Copy, Clone, Debug)]
// Models the possible commands available to this "machine"
enum OpCode {
    Add { r1: Mode, r2: Mode, o: Mode },
    Mul { r1: Mode, r2: Mode, o: Mode },
    Ipt { adr: Mode },
    Opt { o: Mode },
    Jtr { r1: Mode, jmp: Mode },
    Jfl { r1: Mode, jmp: Mode },
    Les { r1: Mode, r2: Mode, o: Mode },
    Eql { r1: Mode, r2: Mode, o: Mode },
    Crl { off: Mode },
    Ext,
}

impl OpCode {
    /// Generate an OpCode from a specific region of memory
    fn from(mem: &[i128], idx: usize) -> Self {
        let instruction = mem[idx];
        let op_code = instruction % 100;
        let mode_spec = instruction as usize / 100;
        let pidx = idx + 1;
        match op_code {
            1 => OpCode::Add {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: Mode::from(mode_spec, mem, pidx, 2),
            },
            2 => OpCode::Mul {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: Mode::from(mode_spec, mem, pidx, 2),
            },
            3 => OpCode::Ipt {
                adr: Mode::from(mode_spec, mem, pidx, 0),
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
                o: Mode::from(mode_spec, mem, pidx, 2),
            },
            8 => OpCode::Eql {
                r1: Mode::from(mode_spec, mem, pidx, 0),
                r2: Mode::from(mode_spec, mem, pidx, 1),
                o: Mode::from(mode_spec, mem, pidx, 2),
            },
            9 => OpCode::Crl {
                off: Mode::from(mode_spec, mem, pidx, 0),
            },
            99 => OpCode::Ext,
            _ => panic!(format!("Unexpected opcode: {}", op_code)),
        }
    }

    // Execute the OpCode against the passed in memory
    fn exec(self, mem: &mut Vec<i128>, io: &impl Io, rel: &mut usize) -> Incr {
        // potentially increase memory size before ops
        match self {
            OpCode::Add { o: adr, .. }
            | OpCode::Mul { o: adr, .. }
            | OpCode::Les { o: adr, .. }
            | OpCode::Eql { o: adr, .. }
            | OpCode::Ipt { adr } => {
                let adr = adr.adr(*rel);
                if adr >= mem.len() {
                    mem.resize(adr + 1, 0);
                }
            }
            _ => {}
        }
        match self {
            OpCode::Ext => return Incr::Exit,
            OpCode::Add { r1, r2, o } => {
                let adr = o.adr(*rel);
                mem[adr] = r1.val(mem, *rel) + r2.val(mem, *rel)
            },
            OpCode::Mul { r1, r2, o } => {
                let adr = o.adr(*rel);
                mem[adr] = r1.val(mem, *rel) * r2.val(mem, *rel)
            },
            OpCode::Ipt { adr } => {
                let adr = adr.adr(*rel);
                mem[adr] = io.read().trim().parse().expect("Failed to parse input")
            }
            OpCode::Opt { o } => io.write(&format!("{}", o.val(mem, *rel))),
            OpCode::Jtr { r1, jmp } => {
                if r1.val(mem, *rel) != 0 {
                    return Incr::Jump(jmp.val(mem, *rel) as usize);
                }
            }
            OpCode::Jfl { r1, jmp } => {
                if r1.val(mem, *rel) == 0 {
                    return Incr::Jump(jmp.val(mem, *rel) as usize);
                }
            }
            OpCode::Les { r1, r2, o } => {
                let adr = o.adr(*rel);
                mem[adr] = if r1.val(mem, *rel) < r2.val(mem, *rel) {
                    1
                } else {
                    0
                }
            }
            OpCode::Eql { r1, r2, o } => {
                let adr = o.adr(*rel);
                mem[adr] = if r1.val(mem, *rel) == r2.val(mem, *rel) {
                    1
                } else {
                    0
                }
            }
            OpCode::Crl { off } => *rel = {
                let res = *rel as isize + off.val(mem, *rel) as isize;
                if res < 0 {
                    panic!("Invalid relative address!")
                }
                res
             } as usize,
        }

        Incr::Offset(self.len())
    }

    fn len(&self) -> usize {
        match self {
            OpCode::Add { .. } | OpCode::Mul { .. } | OpCode::Les { .. } | OpCode::Eql { .. } => 4,
            OpCode::Jtr { .. } | OpCode::Jfl { .. } => 3,
            OpCode::Ipt { .. } | OpCode::Opt { .. } | OpCode::Crl { .. } => 2,
            OpCode::Ext => 1,
        }
    }
}

// Models a simple machine with memory and a program counter
struct Program<'a, T>
where
    T: Io,
{
    mem: &'a mut Vec<i128>,
    ctr: usize,
    rel: usize,
    io: &'a T,
}

impl<'a, T> Program<'a, T>
where
    T: Io,
{
    fn new(mem: &'a mut Vec<i128>, io: &'a T) -> Self {
        Program {
            mem,
            ctr: 0,
            rel: 0,
            io,
        }
    }

    // Process the op codes in memory until an exit opcode is reached
    fn run(mut self) {
        loop {
            let op_code = OpCode::from(self.mem, self.ctr);
            let mut relative = self.rel;

            #[cfg(debug_assertions)]
            {
                let printable_mem = self.mem.iter().enumerate().map(|(i, m)| {
                    if i == self.ctr {
                        format!("*{}:{}*", i, m)
                    } else {
                        format!("{}:{}", i, m)
                    }
                })
                .collect::<Vec<_>>();
                println!(
                    "ctr: {:?}\nrel: {:?}\nmem: [{}]\nop: {:?}\n",
                    self.ctr, self.rel, printable_mem.join(", "), op_code
                );
            }
            self.ctr = match op_code.exec(self.mem, self.io, &mut relative) {
                Incr::Offset(offset) => self.ctr + offset,
                Incr::Jump(address) => address,
                Incr::Exit => break,
            };
            self.rel = relative;
        }
    }
}

pub struct MockIo {
    input: RefCell<VecDeque<String>>,
    output: RefCell<Vec<String>>,
}

impl MockIo {
    pub fn new() -> Self {
        Self {
            input: RefCell::new(VecDeque::new()),
            output: RefCell::new(vec![]),
        }
    }

    pub fn with_input(input: &[&str]) -> Self {
        Self {
            input: RefCell::new(input.iter().map(|&s| s.to_owned()).collect()),
            output: RefCell::new(vec![]),
        }
    }
}

impl Io for MockIo {
    fn read(&self) -> String {
        println!("Reading...");
        self.input
            .borrow_mut()
            .pop_front()
            .expect("Ran out of input")
    }

    fn write(&self, output: &str) {
        self.output.borrow_mut().push(output.to_owned());
    }
}

// struct PipedIo {
//     rx: (String, Receiver<String>),
//     tx: (String, Sender<String>),
// }

// impl PipedIo {
//     fn new(rx: (String, Receiver<String>), tx: (String, Sender<String>)) -> Self {
//         Self { rx, tx }
//     }

//     fn init_input<S: Into<String>>(&mut self, input: S) {
//         self.tx
//             .1
//             .send(input.into())
//             .expect("Failed to initialize input");
//     }

//     fn close(self) -> Receiver<String> {
//         self.rx.1
//     }
// }

// impl Io for PipedIo {
//     fn read(&self) -> String {
//         self.rx.1.recv().expect("Should have gotten input!")
//     }

//     fn write(&self, output: &str) {
//         self.tx
//             .1
//             .send(output.to_owned())
//             .expect("Failed to send data to output");
//     }
// }

// struct Amplifiers<'a> {
//     init_mem: &'a Vec<i128>,
//     phase_settings: &'a [usize],
//     config: Configuration,
// }

// impl<'a> Amplifiers<'a> {
//     fn new(init_mem: &'a Vec<i128>, phase_settings: &'a [usize], config: Configuration) -> Self {
//         Self {
//             init_mem,
//             phase_settings,
//             config,
//         }
//     }

//     fn run(self) -> isize {
//         let mut output = None;
//         match self.config {
//             Configuration::Simple => {
//                 for phase_setting in self.phase_settings.iter() {
//                     let io = MockIo::with_input(&[
//                         &format!("{}\n", phase_setting),
//                         &format!("{}\n", output.unwrap_or_else(|| "0".to_owned())),
//                     ]);

//                     let mut mem = self.init_mem.clone();
//                     let program = Program::new(&mut mem, &io);
//                     program.run();
//                     output = io.output.into_inner().into_iter().next();
//                 }
//             }
//             Configuration::Looped => {
//                 let (mut txs, rxs): (Vec<_>, Vec<_>) = (0..self.phase_settings.len())
//                     .into_iter()

//                     .map(|i| {
//                         let (tx, rx) = mpsc::channel();
//                         let i = format!("{}", i);
//                         ((i.clone(), tx), (i, rx))
//                     })
//                     .unzip();
//                 // tx -> rx
//                 // 0 -> 1
//                 // 1 -> 2
//                 // 2 -> 3
//                 // 3 -> 4
//                 // 4 -> 0
//                 let first_tx = txs.remove(0);
//                 txs.push(first_tx);
//                 let mut pipes = txs
//                     .into_iter()
//                     .zip(rxs.into_iter())
//                     .map(|(tx, rx)| PipedIo::new(rx, tx))
//                     .collect::<Vec<_>>();
//                 for (pipe, phase_setting) in pipes.iter_mut().zip(self.phase_settings.into_iter()) {
//                     pipe.init_input(format!("{}\n", phase_setting));
//                 }
//                 pipes.iter_mut().next().unwrap().init_input("0\n");
//                 let threads = pipes
//                     .into_iter()
//                     .map(|io| {
//                         let mut mem = self.init_mem.clone();
//                         std::thread::spawn(move || {
//                             let program = Program::new(&mut mem, &io);
//                             program.run();
//                             io.close()
//                         })
//                     })
//                     .collect::<Vec<_>>();
//                 for thread in threads {
//                     let rx = thread.join().ok();
//                     let results = rx.map(|rx| rx.iter().collect::<Vec<_>>());
//                     if let Some(mut res) = results {
//                         if res.len() > 0 {
//                             output = Some(res.remove(res.len() - 1));
//                         }
//                     }
//                 }
//             }
//         }

//         output
//             .expect("Expected to have an output!")
//             .parse()
//             .expect("Could not parse output!")
//     }
// }

// fn maximize_amplifiers(init_mem: &Vec<i128>, config: Configuration) -> isize {
//     fn max_amp_util(
//         init_mem: &Vec<i128>,
//         config: Configuration,
//         phase_settings: &mut [usize],
//         size: usize,
//     ) -> isize {
//         if size == 1 {
//             let amplifiers = Amplifiers::new(init_mem, phase_settings, config);
//             return amplifiers.run();
//         }

//         let mut max = 0;
//         for i in 0..size {
//             let res = max_amp_util(init_mem, config, phase_settings, size - 1);
//             if res > max {
//                 max = res;
//             }

//             if size % 2 == 1 {
//                 phase_settings.swap(0, size - 1);
//             } else {
//                 phase_settings.swap(i, size - 1);
//             }
//         }

//         max
//     }

//     let range = match config {
//         Configuration::Simple => 0..5,
//         Configuration::Looped => 5..10,
//     };
//     let mut possible_phase_settings: Vec<_> = range.into_iter().collect();
//     let length = possible_phase_settings.len();
//     max_amp_util(&init_mem, config, &mut possible_phase_settings, length)
// }

fn main() -> Result<(), String> {
    let mut mem = parse("day9/input.txt")?;
    // let io = MockIo::with_input(&["1\n"]);
    let program = Program::new(&mut mem, &RealIo);
    program.run();
    // println!("{}", io.output.borrow()[0]);
    Ok(())
}

fn parse(file_name: &str) -> Result<Vec<i128>, String> {
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
                s.parse::<i128>().ok()
            }
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod test {
    use super::*;

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
            &mut vec![1, 0, 0, 0, 99],
                &[2, 0, 0, 0, 99]);

        // Multiply value at address 3 with value at
        // address 0 and store it in address 3
        #[rustfmt::skip]
        validate_program!(
            &mut vec![2, 3, 0, 3, 99],
                &[2, 3, 0, 6, 99]);

        // Multiply value at address 4 with value itself
        // and store it in address 5
        #[rustfmt::skip]
        validate_program!(
            &mut vec![2, 4, 4, 5, 99, 0],
                &[2, 4, 4, 5, 99, 9801]);

        // Add value at address 1 to itself and store it
        // in address 4 (create mul opcode)
        // Multiply value at address 5 with value at
        // address 6 and store it in address 0
        #[rustfmt::skip]
        validate_program!(
            &mut vec![ 1, 1, 1, 4, 99, 5, 6, 0, 99],
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
            &mut vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
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
            &mut vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
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
            &mut vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
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
            &mut vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
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
            &mut vec![3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
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
            &mut vec![3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
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
            &mut vec![3, 12, 5, 12, 15, 2, 13, 14, 14, 4, 14, 99, -1, 0, 1, 9],
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
            &mut vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
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
            &mut vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
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
            &mut vec![3, 3, 1108, -1, 8, 3, 4, 3, 99],
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
            &mut vec![3, 3, 1108, -1, 8, 3, 4, 3, 99],
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
            &mut vec![3, 3, 1107, -1, 8, 3, 4, 3, 99],
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
            &mut vec![3, 3, 1107, -1, 8, 3, 4, 3, 99],
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
            &mut vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
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
            &mut vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1],
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
            &mut vec![3, 3, 1106, -1, 9, 1101, 0, 1, 12, 4, 12, 99, 0],
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
            &mut vec![3, 3, 1106, -1, 9, 1101, 0, 1, 12, 4, 12, 99, 0],
                &[3, 3, 1106,  1, 9, 1101, 0, 1, 12, 4, 12, 99, 1],
            &["1\n"],
            &["1"],
        );
    }

    #[test]
    fn big_test_lower_prints_999() {
        validate_program_with_io!(
            &mut vec![
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
            &mut vec![
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
            &mut vec![
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

    // #[test]
    // fn amplifiers_case_1_simple() {
    //     let mem = vec![
    //         3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
    //     ];
    //     let amplifiers = Amplifiers::new(&mem, &[4, 3, 2, 1, 0], Configuration::Simple);
    //     let res = amplifiers.run();
    //     assert_eq!(res, 43_210);
    // }

    // #[test]
    // fn amplifiers_case_2_simple() {
    //     let mem = vec![
    //         3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
    //         99, 0, 0,
    //     ];
    //     let amplifiers = Amplifiers::new(&mem, &[0, 1, 2, 3, 4], Configuration::Simple);
    //     let res = amplifiers.run();
    //     assert_eq!(res, 54_321);
    // }

    // #[test]
    // fn amplifiers_case_3_simple() {
    //     let mem = vec![
    //         3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
    //         33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
    //     ];
    //     let amplifiers = Amplifiers::new(&mem, &[1, 0, 4, 3, 2], Configuration::Simple);
    //     let res = amplifiers.run();
    //     assert_eq!(res, 65_210);
    // }

    // #[test]
    // fn amplifiers_case_1_looped() {
    //     let mem = vec![
    //         3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
    //         28, 1005, 28, 6, 99, 0, 0, 5,
    //     ];
    //     let amplifiers = Amplifiers::new(&mem, &[9, 8, 7, 6, 5], Configuration::Looped);
    //     let res = amplifiers.run();
    //     assert_eq!(res, 139_629_729);
    // }

    // #[test]
    // fn amplifiers_case_2_looped() {
    //     let mem = vec![
    //         3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
    //         -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
    //         53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
    //     ];
    //     let amplifiers = Amplifiers::new(&mem, &[9, 7, 8, 5, 6], Configuration::Looped);
    //     let res = amplifiers.run();
    //     assert_eq!(res, 18_216);
    // }

    // #[test]
    // fn max_thrust_case_1() {
    //     let mem = vec![
    //         3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
    //     ];
    //     let res = maximize_amplifiers(&mem, Configuration::Simple);
    //     assert_eq!(res, 43210);
    // }

    // #[test]
    // fn max_thrust_case_2() {
    //     let mem = vec![
    //         3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
    //         99, 0, 0,
    //     ];
    //     let res = maximize_amplifiers(&mem, Configuration::Simple);
    //     assert_eq!(res, 54_321);
    // }

    // #[test]
    // fn max_thrust_case_3() {
    //     let mem = vec![
    //         3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
    //         33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
    //     ];
    //     let res = maximize_amplifiers(&mem, Configuration::Simple);
    //     assert_eq!(res, 65_210);
    // }

    // #[test]
    // fn max_thrust_case_1_looped() {
    //     let mem = vec![
    //         3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
    //         28, 1005, 28, 6, 99, 0, 0, 5,
    //     ];
    //     let res = maximize_amplifiers(&mem, Configuration::Looped);
    //     assert_eq!(res, 139_629_729);
    // }

    // #[test]
    // fn max_thrust_case_2_looped() {
    //     let mem = vec![
    //         3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
    //         -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
    //         53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
    //     ];
    //     let res = maximize_amplifiers(&mem, Configuration::Looped);
    //     assert_eq!(res, 18_216);
    // }

    #[test]
    fn test_op_extra_case_1() {
        validate_program_with_io!(
            &mut vec![109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99],
            &[
                109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 1
            ],
            &[],
            &[
                "109", "1", "204", "-1", "1001", "100", "1", "100", "1008", "100", "16", "101",
                "1006", "101", "0", "99"
            ]
        );
    }

    #[test]
    fn test_op_extra_case_2() {
        validate_program_with_io!(
            &mut vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0],
            &[1102, 34915192, 34915192, 7, 4, 7, 99, 1219070632396864],
            &[],
            &["1219070632396864"]
        );
    }

    #[test]
    fn test_op_extra_case_3() {
        validate_program_with_io!(
            &mut vec![104, 1125899906842624, 99],
            &[104, 1125899906842624, 99],
            &[],
            &["1125899906842624"]
        );
    }
}
