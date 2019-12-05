use std::{fs::File, io::Read};

enum OpCode {
    Add { r1: usize, r2: usize, o: usize },
    Mul { r1: usize, r2: usize, o: usize },
    Exit,
}

impl OpCode {
    fn from(mem: &[usize], idx: usize) -> Self {
        let op_code = mem[idx];
        match op_code {
            1 => OpCode::Add {
                r1: mem[idx + 1],
                r2: mem[idx + 2],
                o: mem[idx + 3],
            },
            2 => OpCode::Mul {
                r1: mem[idx + 1],
                r2: mem[idx + 2],
                o: mem[idx + 3],
            },
            99 => OpCode::Exit,
            _ => panic!(format!("Unexpected opcode: {}", op_code)),
        }
    }

    fn exec(self, mem: &mut [usize]) -> bool {
        match self {
            OpCode::Add { r1, r2, o } => {
                let res = mem[r1] + mem[r2];
                mem[o] = res;
                true
            }
            OpCode::Mul { r1, r2, o } => {
                let res = mem[r1] * mem[r2];
                mem[o] = res;
                true
            }
            OpCode::Exit => false,
        }
    }
}

struct Program<'a> {
    mem: &'a mut [usize],
    ctr: usize,
}

impl<'a> Program<'a> {
    fn new(mem: &'a mut [usize]) -> Self {
        Program { mem, ctr: 0 }
    }

    fn noun(&mut self, noun: usize) {
        self.mem[1] = noun;
    }

    fn verb(&mut self, verb: usize) {
        self.mem[2] = verb;
    }

    fn run(mut self) {
        loop {
            let op_code = OpCode::from(self.mem, self.ctr);
            if !op_code.exec(self.mem) {
                break;
            }

            self.ctr += 4;
        }
    }
}
fn main() -> Result<(), String> {
    let mem = parse()?;
    {
        let mut mem = mem.clone();
        let mut program = Program::new(&mut mem);
        program.noun(12);
        program.verb(2);
        program.run();
        println!("Initial value is: {}", mem[0]);
        println!("Total memory is: {:?}", mem);
    }

    'outer: for noun in 0..100 {
        for verb in 0..100 {
            let mut mem = mem.clone();
            let mut prog = Program::new(&mut mem);
            prog.noun(noun);
            prog.verb(verb);
            prog.run();
            if mem[0] == 19690720 {
                println!("Noun: {}, Verb: {}, Res: {}", noun, verb, 100 * noun + verb);
                break 'outer;
            }
        }
    }
    Ok(())
}

fn parse() -> Result<Vec<usize>, String> {
    let mut file =
        File::open("day2/input.txt").map_err(|e| format!("Failed to open input: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(contents
        .split(",")
        .filter_map(|s| {
            if s == "\n" {
                None
            } else {
                s.parse::<usize>().ok()
            }
        })
        .collect::<Vec<_>>())
}

#[test]
fn test_run() {
    helper(&mut [1, 0, 0, 0, 99], &[2, 0, 0, 0, 99]);
    helper(&mut [2, 3, 0, 3, 99], &[2, 3, 0, 6, 99]);
    helper(&mut [2, 4, 4, 5, 99, 0], &[2, 4, 4, 5, 99, 9801]);
    helper(
        &mut [1, 1, 1, 4, 99, 5, 6, 0, 99],
        &[30, 1, 1, 4, 2, 5, 6, 0, 99],
    );
}

#[cfg(test)]
fn helper(input: &mut [usize], expected: &[usize]) {
    let program = Program::new(input);
    program.run();
    for (l, r) in input.iter().zip(expected.iter()) {
        assert_eq!(l, r);
    }
}
