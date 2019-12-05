use std::ops::RangeInclusive;

trait ValidPassword {
    fn is_valid(&self) -> bool;
    fn is_valid2(&self) -> bool;
}

impl ValidPassword for u32 {
    fn is_valid(&self) -> bool {
        let mut n = *self;
        let mut a = n % 10;

        let mut b = n / 10 % 10;
        if a == 0 || b == 0 {
            return false;
        }

        let mut double = false;
        while n > 0 {
            if a < b {
                return false;
            }

            if a == b && a != 0 {
                double = true;
            }

            n /= 10;
            a = n % 10;
            b = n / 10 % 10;
        }

        double
    }

    fn is_valid2(&self) -> bool {
        let mut bucket = [0; 9];
        let mut n = *self;
        let mut a = n % 10;

        let mut b = n / 10 % 10;
        if a == 0 || b == 0 {
            return false;
        }

        while n > 0 {
            if a < b {
                return false;
            }

            if a == b && a != 0 {
                bucket[(a - 1) as usize] += 1;
            }

            n /= 10;
            a = n % 10;
            b = n / 10 % 10;
        }

        bucket.into_iter().any(|c| *c == 1)
    }
}

#[test]
fn valid_passwords() {
    assert!(111111.is_valid());
    assert!(!647011.is_valid());
    assert!(!223450.is_valid());
    assert!(!123789.is_valid());

    assert!(112233.is_valid2());
    assert!(!123444.is_valid2());
    assert!(111122.is_valid2());
}

struct PasswordCounter {
    range: RangeInclusive<u32>,
    num: u32,
}

impl PasswordCounter {
    fn new(range: RangeInclusive<u32>) -> Self {
        Self {
            num: *range.start(), // cheating but meh
            range,
        }
    }
}

impl Iterator for PasswordCounter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.num.is_valid() && self.num <= *self.range.end() {
            self.num += 1;
        }

        let num = self.num;
        self.num += 1;

        if num > *self.range.end() {
            None
        } else {
            Some(num)
        }
    }
}

struct PasswordCounter2 {
    range: RangeInclusive<u32>,
    num: u32,
}

impl PasswordCounter2 {
    fn new(range: RangeInclusive<u32>) -> Self {
        Self {
            num: *range.start(), // cheating but meh
            range,
        }
    }
}

impl Iterator for PasswordCounter2 {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.num.is_valid2() && self.num <= *self.range.end() {
            self.num += 1;
        }

        let num = self.num;
        self.num += 1;

        if num > *self.range.end() {
            None
        } else {
            Some(num)
        }
    }
}
fn main() {
    let valid_range = 123257..=647015;
    // let real_valid_range = 123333..=599999;
    let counter = PasswordCounter::new(valid_range.clone());
    let total_valid: u32 = counter.into_iter().map(|_| 1).sum();
    println!("Hello, world! {}", total_valid);
    let counter = PasswordCounter2::new(valid_range);
    let total_valid: u32 = counter.into_iter().map(|_| 1).sum();
    println!("Hello, world! {}", total_valid);
}
