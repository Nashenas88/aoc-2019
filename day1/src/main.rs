use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() -> Result<(), String> {
    let file = File::open("day1/input.txt").map_err(|e| format!("{}", e))?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut sum: u64 = 0;
    while let Ok(_) = reader.read_line(&mut buffer) {
        if buffer.len() < 2 {
            break;
        }

        let mass = buffer[0..buffer.len() - 1]
            .parse::<u64>()
            .map_err(|e| format!("Error parsing {}: {}", buffer, e))?;
        sum += all_fuel_for(mass);
        buffer.clear();
    }

    println!("{}", sum);

    Ok(())
}

fn fuel_for(mass: u64) -> u64 {
    (mass / 3).saturating_sub(2)
}

fn all_fuel_for(mass: u64) -> u64 {
    let mut sum = fuel_for(mass);
    let mut last_sum = sum;
    while last_sum > 0 {
        last_sum = fuel_for(last_sum);
        sum += last_sum;
    }

    sum
}

#[test]
fn test_fuel_for() {
    assert_eq!(fuel_for(12), 2);
    assert_eq!(fuel_for(14), 2);
    assert_eq!(fuel_for(1969), 654);
    assert_eq!(fuel_for(100756), 33583);
}

#[test]
fn test_all_fuel_for() {
    assert_eq!(all_fuel_for(14), 2);
    assert_eq!(all_fuel_for(1969), 966);
    assert_eq!(all_fuel_for(100756), 50346);
}
