use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{prelude::*, BufReader},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn manhattan_distance(&self) -> u64 {
        self.x.abs() as u64 + self.y.abs() as u64
    }

    fn flat_distance_to(&self, other: &Self) -> u64 {
        (self.x - other.x).abs() as u64 + (self.y - other.y).abs() as u64
    }
}

enum Polarity {
    Vertical,
    Horizontal,
}

#[allow(dead_code)]
impl Polarity {
    fn is_horizontal(&self) -> bool {
        match self {
            Polarity::Horizontal => true,
            _ => false,
        }
    }

    fn is_vertical(&self) -> bool {
        match self {
            Polarity::Vertical => true,
            _ => false,
        }
    }
}

struct Bounds {
    low: i64,
    high: i64,
    bar: i64,
}

struct Segment(Point, Point);

impl Segment {
    fn polarity_and_bounds(&self) -> (Polarity, Bounds) {
        if self.0.x == self.1.x {
            if self.0.y < self.1.y {
                (
                    Polarity::Vertical,
                    Bounds {
                        low: self.0.y,
                        high: self.1.y,
                        bar: self.0.x,
                    },
                )
            } else {
                (
                    Polarity::Vertical,
                    Bounds {
                        low: self.1.y,
                        high: self.0.y,
                        bar: self.0.x,
                    },
                )
            }
        } else {
            if self.0.x < self.1.x {
                (
                    Polarity::Horizontal,
                    Bounds {
                        low: self.0.x,
                        high: self.1.x,
                        bar: self.0.y,
                    },
                )
            } else {
                (
                    Polarity::Horizontal,
                    Bounds {
                        low: self.1.x,
                        high: self.0.x,
                        bar: self.0.y,
                    },
                )
            }
        }
    }

    fn crosses(&self, other: &Segment) -> Option<(Point, u64, u64)> {
        let point = match (self.polarity_and_bounds(), other.polarity_and_bounds()) {
            ((Polarity::Horizontal, ..), (Polarity::Horizontal, ..))
            | ((Polarity::Vertical, ..), (Polarity::Vertical, ..)) => None,
            ((Polarity::Vertical, v_bounds), (Polarity::Horizontal, h_bounds))
            | ((Polarity::Horizontal, h_bounds), (Polarity::Vertical, v_bounds)) => {
                if h_bounds.bar <= v_bounds.low
                    || h_bounds.bar >= v_bounds.high
                    || v_bounds.bar <= h_bounds.low
                    || v_bounds.bar >= h_bounds.high
                {
                    None
                } else {
                    Some(Point {
                        x: v_bounds.bar,
                        y: h_bounds.bar,
                    })
                }
            }
        };

        point.map(|p| (p, self.0.flat_distance_to(&p), other.0.flat_distance_to(&p)))
    }

    fn length(&self) -> u64 {
        match self.polarity_and_bounds() {
            (_, Bounds { low, high, .. }) => (high - low) as u64,
        }
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{{({}, {}) - ({}, {})}}",
            self.0.x, self.0.y, self.1.x, self.1.y
        )
    }
}

#[derive(Copy, Clone, Debug)]
enum Route {
    Up(u32),
    Down(u32),
    Left(u32),
    Right(u32),
}

#[derive(Debug)]
enum Either<T, U> {
    A(T),
    B(U),
}

impl FromStr for Route {
    type Err = Either<char, ParseIntError>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first_char = s.as_bytes()[0] as char;
        let num = s[1..].parse().map_err(Either::B)?;
        match first_char {
            'U' => Ok(Route::Up(num)),
            'D' => Ok(Route::Down(num)),
            'L' => Ok(Route::Left(num)),
            'R' => Ok(Route::Right(num)),
            _ => Err(Either::A(first_char)),
        }
    }
}

struct Runner {
    path: Vec<Segment>,
    cursor: Point,
}

impl Runner {
    fn new() -> Self {
        Self {
            path: vec![],
            cursor: Point { x: 0, y: 0 },
        }
    }

    fn follow(&mut self, route: Route) {
        let next = match route {
            Route::Up(u) => Point {
                y: self.cursor.y + u as i64,
                ..self.cursor
            },
            Route::Down(d) => Point {
                y: self.cursor.y - d as i64,
                ..self.cursor
            },
            Route::Left(l) => Point {
                x: self.cursor.x - l as i64,
                ..self.cursor
            },
            Route::Right(r) => Point {
                x: self.cursor.x + r as i64,
                ..self.cursor
            },
        };

        let segment = Segment(self.cursor, next);
        self.path.push(segment);
        self.cursor = next;
    }

    fn finish(self) -> Vec<Segment> {
        self.path
    }
}

fn run(route1: Vec<Route>, route2: Vec<Route>) -> (u64, u64) {
    let mut runner1 = Runner::new();
    for route in &route1 {
        runner1.follow(*route);
    }
    let segments1 = runner1.finish();

    let mut runner2 = Runner::new();
    for route in &route2 {
        runner2.follow(*route);
    }
    let segments2 = runner2.finish();

    // This whole section could definitely be optimized...
    // O(n*m)
    #[cfg(not(feature = "optimized"))]
    {
        let mut crosses = vec![];
        let mut cross_distances = HashMap::<Point, u64>::new();
        let mut s1sum = 0;
        for s1 in &segments1 {
            let mut s2sum = 0;
            for s2 in &segments2 {
                if let Some((p, s1dist, s2dist)) = s1.crosses(s2) {
                    crosses.push(p);
                    if !cross_distances.contains_key(&p) {
                        cross_distances.insert(p, s1sum + s1dist + s2sum + s2dist);
                    }
                }

                s2sum += s2.length();
            }

            s1sum += s1.length();
        }

        let min_manhattan = crosses
            .into_iter()
            .map(|p| p.manhattan_distance())
            .min()
            .unwrap();
        let min_sum_dist = cross_distances.into_iter().map(|(_, v)| v).min().unwrap();
        (min_manhattan, min_sum_dist)
    }

    // optimized
    // O(n log n + m log m)

    #[cfg(feature = "optimized")]
    {
        struct ComputeData {
            segment: Segment,
            polarity: Polarity,
            bounds: Bounds,
            previous_length: u64,
        }
        // First we compute the lengths to get to each segment
        // and store them together
        fn compute_data(seg: Vec<Segment>) -> Vec<ComputeData> {
            let mut length = 0;
            seg.into_iter()
                .map(|segment| {
                    let next_length = segment.length();
                    let (polarity, bounds) = segment.polarity_and_bounds();
                    let result = ComputeData {
                        segment,
                        polarity,
                        bounds,
                        previous_length: length,
                    };
                    length += next_length;
                    result
                })
                .collect()
        }
        let data1 = compute_data(segments1);
        let data2 = compute_data(segments2);

        // Next we split each segment into horizontal and vertical
        // vectors, then sort them according to their horizontal component
        fn partition_and_sort(seg: &[ComputeData]) -> (Vec<&ComputeData>, Vec<&ComputeData>) {
            let (mut horizontals, mut verticals): (Vec<_>, Vec<_>) =
                seg.iter().partition(|data| data.polarity.is_horizontal());
            horizontals.sort_by_key(|data| data.segment.0.x);
            verticals.sort_by_key(|data| data.segment.0.x);
            (horizontals, verticals)
        }
        let (h1s, v1s) = partition_and_sort(&data1);
        let (h2s, v2s) = partition_and_sort(&data2);

        // now we can iterate over each horizontal and vertical pair in O(n+m)
        fn find_manhattan_and_min_sum_distances(
            horizontals: &[&ComputeData],
            verticals: &[&ComputeData],
        ) -> (u64, u64) {
            let mut h_iter = horizontals.iter();
            let mut v_iter = verticals.iter();
            let h_item = h_iter.next();
            let v_item = v_iter.next();

            // huh? Why the inner stuff here?
            // We might run into cases where there are multiple horizontals
            // and verticals crossing each other (think of the pound sign -> #).
            // Iterating to the next vertical or horizontal after a successful
            // intersection would be incorrect. Here, I've chosen to clone the
            // verticals' iterator and run nested checks against the following
            // vertical segments until they extend past the current horizontal
            // segment. After that nested scan is complete, we could then move
            // on to the next horizontal segment in the outer loop.          ^
            // P.S. would you look at that alignment!? ----------------------^
            fn inner_find<'a>(
                mut h_item: Option<&'a &'a ComputeData>,
                mut v_item: Option<&'a &'a ComputeData>,
                h_iter: &mut impl Iterator<Item = &'a &'a ComputeData>,
                v_iter: &mut (impl Iterator<Item = &'a &'a ComputeData> + Clone),
                nested: bool,
            ) -> (u64, u64) {
                let mut min_manhattan = std::u64::MAX;
                let mut min_sum = std::u64::MAX;

                while let (Some(h_data), Some(v_data)) = (h_item, v_item) {
                    // In these cases, the vertical segment doesn't intersect, and
                    // there still might be other vertical segments that could
                    // intersect with this horizontal segment (due to sorting by x)
                    // so just move to the next vertical segment.
                    if v_data.bounds.bar <= h_data.bounds.low
                        || h_data.bounds.bar <= v_data.bounds.low
                        || h_data.bounds.bar >= v_data.bounds.high
                    {
                        v_item = v_iter.next();
                        continue;
                    }

                    // Here the vertical segment is beyond the current horizontal
                    // segment. Given that we sort by x, no more vertical
                    // segments will intersect with this horizontal segment. Move
                    // to the next horizontal segment. If we're in the nested
                    // loop, then just exit. The outer loop will increment for us.
                    if v_data.bounds.bar >= h_data.bounds.high {
                        if nested {
                            return (min_manhattan, min_sum);
                        }

                        h_item = h_iter.next();
                        continue;
                    }

                    let p = Point {
                        x: v_data.bounds.bar,
                        y: h_data.bounds.bar,
                    };

                    let p_manhattan = p.manhattan_distance();
                    if p_manhattan < min_manhattan {
                        min_manhattan = p_manhattan;
                    }

                    let p_min_sum = h_data.previous_length
                        + h_data.segment.0.flat_distance_to(&p)
                        + v_data.previous_length
                        + v_data.segment.0.flat_distance_to(&p);
                    if p_min_sum < min_sum {
                        min_sum = p_min_sum;
                    }

                    if nested {
                        v_item = v_iter.next();
                        continue;
                    }

                    let (inner_manhattan, inner_min_sum) =
                        inner_find(h_item, v_item, h_iter, &mut v_iter.clone(), true);
                    if inner_manhattan < min_manhattan {
                        min_manhattan = inner_manhattan;
                    }
                    if inner_min_sum < min_sum {
                        min_sum = inner_min_sum;
                    }
                    h_item = h_iter.next();
                }

                (min_manhattan, min_sum)
            }

            inner_find(h_item, v_item, &mut h_iter, &mut v_iter, false)
        }

        let (manhattan_a, min_sum_a) = find_manhattan_and_min_sum_distances(&h1s, &v2s);
        let (manhattan_b, min_sum_b) = find_manhattan_and_min_sum_distances(&h2s, &v1s);

        (manhattan_a.min(manhattan_b), min_sum_a.min(min_sum_b))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! segments_cross {
        {(($x1:expr, $y1:expr), ($x2:expr, $y2:expr)) <-|-> (($x3:expr, $y3:expr), ($x4:expr, $y4:expr)) @ ($c1:expr, $c2:expr)} => {
            let segment1 = Segment(Point{x: $x1, y: $y1}, Point{x: $x2, y: $y2});
            let segment2 = Segment(Point{x: $x3, y: $y3}, Point{x: $x4, y: $y4});
            let cross = segment1.crosses(&segment2);
            assert!(cross.is_some());
            let (cross, ..) = cross.unwrap();
            assert_eq!(cross.x, $c1);
            assert_eq!(cross.y, $c2);
        };
        {(($x1:expr, $y1:expr), ($x2:expr, $y2:expr)) <---> (($x3:expr, $y3:expr), ($x4:expr, $y4:expr))} => {
            let segment1 = Segment(Point{x: $x1, y: $y1}, Point{x: $x2, y: $y2});
            let segment2 = Segment(Point{x: $x3, y: $y3}, Point{x: $x4, y: $y4});
            assert!(segment1.crosses(&segment2).is_none());
        };
    }

    #[test]
    fn segments_cross() {
        // two vertical
        segments_cross!((( 0,  1), ( 0, -1)) <---> (( 0,  2), ( 0, -2)));
        // two horizontal
        segments_cross!((( 1,  0), (-1,  0)) <---> (( 2,  0), (-2,  0)));
        // too far left
        segments_cross!(((-1,  0), ( 1,  0)) <---> ((-2,  1), (-2, -1)));
        // too far right
        segments_cross!(((-1,  0), ( 1,  0)) <---> (( 2,  1), ( 2, -1)));
        // too far up
        segments_cross!((( 2, -1), ( 2,  1)) <---> ((-1,  0), ( 1,  0)));
        // too far down
        segments_cross!(((-2, -1), (-2,  1)) <---> ((-1,  0), ( 1,  0)));
        // cross
        segments_cross!(((-1,  0), ( 1,  0)) <-|-> (( 0, -1), ( 0,  1)) @ (0, 0));
        // on-edge should not cross
        segments_cross!(((-1,  0), ( 1,  0)) <---> (( 1, -1), ( 1,  1)));
    }

    macro_rules! assert_segments_eq {
        ($seg:expr, {($x1:expr, $y1:expr), ($x2:expr, $y2:expr)}) => {
            assert_eq!($seg.0.x, $x1);
            assert_eq!($seg.0.y, $y1);
            assert_eq!($seg.1.x, $x2);
            assert_eq!($seg.1.y, $y2);
        };
    }

    #[test]
    fn runner() {
        let mut runner = Runner::new();
        runner.follow(Route::Up(4));
        runner.follow(Route::Right(4));
        runner.follow(Route::Down(4));
        runner.follow(Route::Left(4));
        let path = runner.finish();
        assert_segments_eq!(path[0], {(0, 0), (0, 4)});
        assert_segments_eq!(path[1], {(0, 4), (4, 4)});
        assert_segments_eq!(path[2], {(4, 4), (4, 0)});
        assert_segments_eq!(path[3], {(4, 0), (0, 0)});
    }

    macro_rules! route_vec {
        (@route R $num:expr) => {
            Route::Right($num)
        };
        (@route L $num:expr) => {
            Route::Left($num)
        };
        (@route U $num:expr) => {
            Route::Up($num)
        };
        (@route D $num:expr) => {
            Route::Down($num)
        };
        ([$($route:ident $num:expr,)+]) => {
            vec![$(route_vec!(@route $route $num),)+]
        };
    }

    macro_rules! distance_of {
        ([$($route1:ident $num1:expr),+], [$($route2:ident $num2:expr),+] = $dist:expr) => {
            let route1 = route_vec!([$($route1 $num1,)+]);
            let route2 = route_vec!([$($route2 $num2,)+]);
            let (dist, _) = run(route1, route2);
            assert_eq!(dist, $dist);
        }
    }

    macro_rules! min_sum_distance_of {
        ([$($route1:ident $num1:expr),+], [$($route2:ident $num2:expr),+] = $dist:expr) => {
            let route1 = route_vec!([$($route1 $num1,)+]);
            let route2 = route_vec!([$($route2 $num2,)+]);
            let (_, dist) = run(route1, route2);
            assert_eq!(dist, $dist);
        }
    }

    #[test]
    fn manhattan_distance() {
        distance_of!(
            [R 75, D 30, R 83, U 83, L 12, D 49, R 71, U  7, L 72],
            [U 62, R 66, U 55, R 34, D 71, R 55, D 58, R 83] = 159);
        distance_of!(
            [R 98, U 47, R 26, D 63, R 33, U 87, L 62, D 20, R 33, U 53, R 51],
            [U 98, R 91, D 20, R 16, D 67, R 40, U  7, R 15, U  6, R  7] = 135);
    }

    #[test]
    fn min_sum_distance() {
        min_sum_distance_of!(
            [R 75, D 30, R 83, U 83, L 12, D 49, R 71, U  7, L 72],
            [U 62, R 66, U 55, R 34, D 71, R 55, D 58, R 83] = 610);
        min_sum_distance_of!(
            [R 98, U 47, R 26, D 63, R 33, U 87, L 62, D 20, R 33, U 53, R 51],
            [U 98, R 91, D 20, R 16, D 67, R 40, U  7, R 15, U  6, R  7] = 410);
    }
}

fn parse_line(reader: &mut BufReader<File>, buffer: &mut String) -> Result<Vec<Route>, String> {
    reader
        .read_line(buffer)
        .map_err(|e| format!("Could not read file: {}", e))?;
    Ok(buffer
        .split(",")
        .filter_map(|s| {
            if s.len() <= 1 {
                return None;
            }

            let mut s = s;
            if s.ends_with("\n") {
                s = s.split("\n").next().unwrap();
            }

            s.parse().ok()
        })
        .collect::<Vec<_>>())
}

fn main() -> Result<(), String> {
    let file = File::open("day3/input.txt").map_err(|e| format!("Could not open input: {}", e))?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let route1 = parse_line(&mut reader, &mut buffer)?;
    buffer.clear();
    let route2 = parse_line(&mut reader, &mut buffer)?;
    let (min_manhattan, min_sum_dist) = run(route1, route2);
    println!("Manhattan distance: {}", min_manhattan);
    println!("Min sum distance: {}", min_sum_dist);
    Ok(())
}
