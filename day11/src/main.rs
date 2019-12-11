use int_code_emulator::{parse, PipedIo, Program};
use std::{
    collections::{hash_map::Entry, HashMap},
    isize,
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Copy, Clone)]
enum Color {
    Black,
    White,
}

trait IoRepr {
    fn repr(&self) -> &'static str;
}

impl IoRepr for Color {
    fn repr(&self) -> &'static str {
        match self {
            Color::Black => "0",
            Color::White => "1",
        }
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Color, ()> {
        match s.trim() {
            "0" => Ok(Color::Black),
            "1" => Ok(Color::White),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
enum Turn {
    Left,
    Right,
}

impl FromStr for Turn {
    type Err = ();

    fn from_str(s: &str) -> Result<Turn, ()> {
        match s.trim() {
            "0" => Ok(Turn::Left),
            "1" => Ok(Turn::Right),
            _ => Err(()),
        }
    }
}

impl Direction {
    fn turn(&mut self, turn: Turn) {
        *self = match (*self, turn) {
            (Direction::Up, Turn::Left) | (Direction::Down, Turn::Right) => Direction::Left,
            (Direction::Up, Turn::Right) | (Direction::Down, Turn::Left) => Direction::Right,
            (Direction::Left, Turn::Right) | (Direction::Right, Turn::Left) => Direction::Up,
            (Direction::Right, Turn::Right) | (Direction::Left, Turn::Left) => Direction::Down,
        }
    }

    fn next(&self, (x, y): (isize, isize)) -> (isize, isize) {
        match self {
            Direction::Up => (x, y + 1),
            Direction::Down => (x, y - 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        }
    }
}

struct Painter<'a> {
    tx: Sender<String>,
    rx: Receiver<String>,
    program: Option<Program<'a, PipedIo>>,
}

impl<'a> Painter<'a> {
    fn new(mem: &'a mut Vec<i128>) -> Painter<'a> {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let io1 = PipedIo::new((String::new(), rx1), (String::new(), tx2));
        Self {
            program: Some(Program::new(mem, io1)),
            rx: rx2,
            tx: tx1,
        }
    }

    fn track_painter(
        init: Color,
        rx: Receiver<String>,
        tx: Sender<String>,
    ) -> HashMap<(isize, isize), Color> {
        let mut direction = Direction::Up;
        let mut visited = HashMap::new();
        let mut pos: (isize, isize) = (0, 0);
        tx.send(format!("{}\n", init.repr()))
            .expect("Failed to send initializing color");
        while let Ok(val) = rx.recv() {
            let color = val.parse::<Color>().unwrap();
            let turn = rx.recv().expect("turn").parse::<Turn>().unwrap();
            visited
                .entry(pos)
                .and_modify(|c| *c = color)
                .or_insert(color);
            direction.turn(turn);
            pos = direction.next(pos);
            let to_send = visited.get(&pos).map(|c| c.repr()).unwrap_or("0");
            if let Err(_) = tx.send(format!("{}\n", to_send)) {
                break;
            }
        }

        visited
    }

    fn run(mut self, init: Color) -> HashMap<(isize, isize), Color> {
        let program = self.program.take().unwrap();
        let (rx, tx) = (self.rx, self.tx);
        let thread = std::thread::spawn(move || Self::track_painter(init, rx, tx));
        program.run();
        thread.join().expect("Could not join thread")
    }
}

fn draw(map: HashMap<(isize, isize), Color>) {
    let (min_x, max_x, min_y, max_y) = map.iter().fold(
        (isize::MAX, isize::MIN, isize::MAX, isize::MIN),
        |(mut min_x, mut max_x, mut min_y, mut max_y), (&(x, y), _)| {
            if x < min_x {
                min_x = x;
            } else if x > max_x {
                max_x = x;
            }

            if y < min_y {
                min_y = y;
            } else if y > max_y {
                max_y = y;
            }

            (min_x, max_x, min_y, max_y)
        },
    );

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let draw = match map.get(&(x, y)) {
                Some(c) => match c {
                    Color::Black => " ",
                    Color::White => "#",
                },
                None => " ",
            };
            print!("{}", draw);
        }
        println!();
    }
}

fn main() {
    let mut mem = parse("day11/input.txt").expect("Failed to parse int code");
    let mut mem_clone = mem.clone();
    let painter = Painter::new(&mut mem_clone);
    let painted = painter.run(Color::Black);
    println!("Total locations painted: {}", painted.len());

    let painter = Painter::new(&mut mem);
    let painted = painter.run(Color::White);
    draw(painted);
}
