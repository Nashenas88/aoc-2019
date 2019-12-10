use std::{
    fs::File,
    io::Read,
};

type Row = Vec<u8>;

struct Layer {
    rows: Vec<Row>,
}

fn parse(input: &str, width: usize, height: usize) -> Vec<Layer> {
    input
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|chunk| chunk.into_iter().map(|&u| u).collect::<Vec<_>>())
        .collect::<Vec<Vec<_>>>()
        .chunks(height)
        .map(|chunk| Layer {
            rows: chunk.into_iter().map(|v| v.clone()).collect(),
        })
        .collect()
}

fn main() {
    let width = 25;
    let height = 6;
    let mut buffer = String::new();
    let mut file = File::open("day8/input.txt").expect("Failed to open file");
    file.read_to_string(&mut buffer)
        .expect("Failed to read from file");
    let layers = parse(buffer.trim(), width, height);
    let layer = layers
        .iter()
        .map(|l| {
            (
                l.rows
                    .iter()
                    .map(|r| r.iter().filter(|&&i| i == 0).count())
                    .sum::<usize>(),
                l,
            )
        })
        .min_by_key(|&(c, _)| c)
        .map(|(_, l)| l)
        .expect("Should have had something...");
    let l1s = layer.rows.iter().map(|r| r.iter().filter(|&&i| i == 1).count()).sum::<usize>();
    let l2s = layer.rows.iter().map(|r| r.iter().filter(|&&i| i == 2).count()).sum::<usize>();
    println!("Hello, world! {}", l1s * l2s);

    let mut final_layer: Vec<Vec<Option<u8>>> = vec![vec![None; width]; height];
    for layer in &layers {
        for (r, row) in layer.rows.iter().enumerate() {
            for (c, col) in row.iter().enumerate() {
                if *col == 2 {
                    continue;
                }

                if let pixel @ None = &mut final_layer[r][c] {
                    *pixel = Some(*col);
                }
            }
        }
    }
    for row in final_layer {
        for col in row {
            print!("{}", match col {
                Some(v) if v == 1 => format!("{}", v),
                Some(v) if v == 0 => " ".to_owned(),
                None => "2".to_owned(),
                _ => "3".to_owned(),
            })
        }
        println!();
    }
}

#[test]
fn parse_test() {
    let layers = parse("123456789012", 3, 2);
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].rows.len(), 2);
    assert_eq!(&layers[0].rows[0], &[1, 2, 3]);
    assert_eq!(&layers[0].rows[1], &[4, 5, 6]);
    assert_eq!(layers[1].rows.len(), 2);
    assert_eq!(&layers[1].rows[0], &[7, 8, 9]);
    assert_eq!(&layers[1].rows[1], &[0, 1, 2]);
}
