use ordered_float::NotNan;
use std::{collections::BTreeMap, fs::File, io::Read};

type Field = Vec<Vec<bool>>;

fn parse(asteroid_field: &str) -> Field {
    let mut vec = vec![];
    for line in asteroid_field.lines() {
        vec.push(line.trim().chars().map(|c| c == '#').collect());
    }
    vec
}

fn filter_asteroids(field: Vec<Vec<bool>>) -> Vec<(usize, usize)> {
    field
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, &b)| b)
                .map(move |(x, _)| (x, y))
        })
        .flatten()
        .collect()
}

fn build_index(
    (x, y): (usize, usize),
    asteroids: &[(usize, usize)],
    best_to_beat: usize,
) -> Option<(usize, BTreeMap<NotNan<f64>, Vec<(usize, usize, usize)>>)> {
    let mut aligned_map = BTreeMap::new();
    for &(a, b) in asteroids {
        if x == a && y == b {
            continue;
        }

        let xdiff = a as isize - x as isize;
        let ydiff = b as isize - y as isize;
        let dist_squared = (xdiff * xdiff + ydiff * ydiff) as usize;

        let angle = NotNan::new((ydiff as f64).atan2(xdiff as f64)).expect("craaap");
        let to_add = (a, b, dist_squared);
        aligned_map
            .entry(angle)
            .and_modify(|dist_vec: &mut Vec<(usize, usize, usize)>| {
                dist_vec.push(to_add);
            })
            .or_insert(vec![to_add]);
    }

    let count = aligned_map.len();
    if count > best_to_beat {
        Some((count, aligned_map))
    } else {
        None
    }
}

fn analyze(
    field: Field,
) -> (
    (usize, usize),
    usize,
    BTreeMap<NotNan<f64>, Vec<(usize, usize, usize)>>,
) {
    let asteroids_with_indices = filter_asteroids(field);
    let mut best = ((0, 0), 0, BTreeMap::new());
    for coords in &asteroids_with_indices {
        if let Some((count, map)) = build_index(*coords, &asteroids_with_indices, best.1) {
            best = (*coords, count, map);
        }
    }

    best
}

fn laser_sweep(
    mut map: BTreeMap<NotNan<f64>, Vec<(usize, usize, usize)>>,
    shot_count: usize,
) -> (usize, usize) {
    // sort the distances so the closer asteroids are iterated over first
    for (_, vec) in map.iter_mut() {
        vec.sort_by_key(|tup| tup.2);
    }

    let mut ordered_asteroids: Vec<_> = map
        .into_iter()
        .map(|(k, v)| {
            let angle = change_angle(k.to_degrees());
            let k = NotNan::new(angle).unwrap();
            (k, v)
        })
        .collect();
    ordered_asteroids.sort_by_key(|t| t.0);

    let mut count = 0;
    while count < shot_count && ordered_asteroids.len() > 0 {
        for (_, vec) in &mut ordered_asteroids {
            count += 1;
            let (x, y, _) = vec.remove(0);
            if count == shot_count {
                return (x, y);
            }
        }
        ordered_asteroids.retain(|(_, vec)| vec.len() > 0);
    }

    (0, 0)
}

fn main() {
    let field = {
        let mut file = File::open("day10/input.txt").expect("Failed to open input file");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to read file");
        parse(&buffer)
    };
    let ((x, y), count, map) = analyze(field);
    let shot_coord = laser_sweep(map, 200);
    println!(
        "Best asteroid is at ({}, {}) and can see {} other asteroids",
        x, y, count
    );
    println!(
        "200th asteroid cleared from that pos: ({}, {})",
        shot_coord.0, shot_coord.1
    );
}

fn change_angle(angle: f64) -> f64 {
    let mut angle = angle;
    angle += 90.0;
    if angle >= 360.0 {
        angle -= 360.0;
    } else if angle < 0.0 {
        angle += 360.0;
    }

    angle
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_feq {
        ($left:expr, $right:expr) => {
            let left = $left;
            let right = $right;
            assert!(
                (left - right).abs() < std::f64::EPSILON,
                "{} is not close enough to {}",
                left,
                right
            );
        };
    }

    #[test]
    fn change_angle_tests() {
        assert_feq!(0.0, change_angle(270.0));
        assert_feq!(90.0, change_angle(0.0));
        assert_feq!(180.0, change_angle(90.0));
        assert_feq!(270.0, change_angle(180.0));
    }

    macro_rules! validate_field {
        ({$field:expr} => $count:expr, ($x:expr, $y:expr)) => {
            let field = parse($field);
            let ((x, y), count, _) = analyze(field);
            assert_eq!($x, x, "X-coordinate mismatch");
            assert_eq!($y, y, "Y-coordinate mismatch");
            assert_eq!($count, count, "Count mismatch");
        };
    }

    #[test]
    fn case_1() {
        validate_field! {
            {
r"......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####"
            } => 33, (5, 8)
        };
    }

    #[test]
    fn case_2() {
        validate_field! {
            {
r"#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###."
            } => 35, (1, 2)
        };
    }

    #[test]
    fn case_3() {
        validate_field! {
            {
r".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#.."
            } => 41, (6, 3)
        };
    }

    #[test]
    fn case_4() {
        validate_field! {
            {
r".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##"
            } => 210, (11, 13)
        };
    }

    #[test]
    fn case_sweep() {
        let field = {
            r".#..##.###...#######
            ##.############..##.
            .#.######.########.#
            .###.#######.####.#.
            #####.##.#.##.###.##
            ..#####..#.#########
            ####################
            #.####....###.#.#.##
            ##.#################
            #####.##.###..####..
            ..######..##.#######
            ####.##.####...##..#
            .#####..#.######.###
            ##...#.##########...
            #.##########.#######
            .####.#.###.###.#.##
            ....##.##.###..#####
            .#.#.###########.###
            #.#.#.#####.####.###
            ###.##.####.##.#..##"
        };

        let field = parse(field);
        let asteroids = filter_asteroids(field);
        let (_, map) = build_index((11, 13), &asteroids, 0).expect("Need at least 1...");
        let coords = laser_sweep(map.clone(), 1);
        assert_eq!(coords, (11, 12));
        let coords = laser_sweep(map.clone(), 2);
        assert_eq!(coords, (12, 1));
        let coords = laser_sweep(map.clone(), 3);
        assert_eq!(coords, (12, 2));
        let coords = laser_sweep(map.clone(), 10);
        assert_eq!(coords, (12, 8));
        let coords = laser_sweep(map.clone(), 20);
        assert_eq!(coords, (16, 0));
        let coords = laser_sweep(map.clone(), 50);
        assert_eq!(coords, (16, 9));
        let coords = laser_sweep(map.clone(), 100);
        assert_eq!(coords, (10, 16));
        let coords = laser_sweep(map.clone(), 199);
        assert_eq!(coords, (9, 6));
        let coords = laser_sweep(map.clone(), 200);
        assert_eq!(coords, (8, 2));
        let coords = laser_sweep(map.clone(), 201);
        assert_eq!(coords, (10, 9));
        let coords = laser_sweep(map.clone(), 299);
        assert_eq!(coords, (11, 1));
    }
}
