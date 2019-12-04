use std::collections::{HashMap, HashSet};
use std::io::{stdin, BufRead};
use std::str::FromStr;

enum Direction {
    Right,
    Left,
    Down,
    Up,
}

impl FromStr for Direction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "D" => Ok(Self::Down),
            "L" => Ok(Self::Left),
            "U" => Ok(Self::Up),
            "R" => Ok(Self::Right),
            _ => Err("Invalid direction character"),
        }
    }
}

struct Segment {
    direction: Direction,
    distance: i32,
}

impl FromStr for Segment {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction = s[..1].parse()?;
        let distance = s[1..].parse().map_err(|_| "invalid number")?;
        Ok(Self {
            direction,
            distance,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Point {
    x: i32,
    y: i32,
}

struct Points<I> {
    segments: I,
    position: Point,
    direction: Option<Direction>,
    remaining: i32,
}

impl<I> Iterator for Points<I>
where
    I: Iterator<Item = Segment>,
{
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        let direction = match self.direction.take() {
            Some(d) => d,
            None => match self.segments.next() {
                Some(segment) => {
                    self.remaining = segment.distance;
                    segment.direction
                }
                None => return None,
            },
        };
        let position = match direction {
            Direction::Up => Point {
                y: self.position.y + 1,
                ..self.position
            },
            Direction::Down => Point {
                y: self.position.y - 1,
                ..self.position
            },
            Direction::Left => Point {
                x: self.position.x - 1,
                ..self.position
            },
            Direction::Right => Point {
                x: self.position.x + 1,
                ..self.position
            },
        };

        self.remaining -= 1;
        if self.remaining > 0 {
            self.direction = Some(direction);
        }
        self.position = position.clone();

        Some(position)
    }
}

impl<I: Iterator<Item = Segment>> Points<I> {
    fn new(segments: I) -> Self {
        Self {
            segments,
            position: Point { x: 0, y: 0 },
            direction: None,
            remaining: 0,
        }
    }
}

fn find_minimum_distance(first: &str, second: &str) -> Result<u32, &'static str> {
    let first_segments = first
        .split(",")
        .map(Segment::from_str)
        .collect::<Result<Vec<Segment>, _>>()?;

    let first_set = Points::new(first_segments.into_iter()).collect::<HashSet<Point>>();

    let second_segments = second
        .split(",")
        .map(Segment::from_str)
        .collect::<Result<Vec<Segment>, _>>()?;

    Ok(Points::new(second_segments.into_iter())
        .filter(|point| first_set.contains(point))
        .map(|point| point.x.abs() as u32 + point.y.abs() as u32)
        .min()
        .unwrap_or(0))
}

fn find_minimum_combined_steps(first: &str, second: &str) -> Result<u32, &'static str> {
    let first_segments = first
        .split(",")
        .map(Segment::from_str)
        .collect::<Result<Vec<Segment>, _>>()?;

    // We aren't emitting the point (0, 0), so a step index is one
    // lower than expected
    let first_set = Points::new(first_segments.into_iter()).enumerate().fold(
        HashMap::new(),
        |mut map, (index, point)| {
            map.entry(point).or_insert(index + 1);
            map
        },
    );

    let second_segments = second
        .split(",")
        .map(Segment::from_str)
        .collect::<Result<Vec<Segment>, _>>()?;

    Ok(Points::new(second_segments.into_iter())
        .enumerate()
        .filter(|(_, point)| first_set.contains_key(&point))
        .map(|(index, point)| 1 + index as u32 + *first_set.get(&point).unwrap() as u32)
        .min()
        .unwrap_or(0))
}

fn main() {
    let input = stdin()
        .lock()
        .lines()
        .collect::<std::io::Result<Vec<String>>>()
        .unwrap();

    if input.len() != 2 {
        eprintln!("Got wrong number of input lines");
        return;
    }
    let result = find_minimum_distance(&input[0], &input[1]).unwrap();
    println!("Puzzle 1 - {}", result);

    let result = find_minimum_combined_steps(&input[0], &input[1]).unwrap();
    println!("Puzzle 2 - {}", result);
}

#[cfg(test)]
mod test {
    use super::{find_minimum_combined_steps, find_minimum_distance};

    #[test]
    fn test_minimum_distance_case_1() {
        assert_eq!(find_minimum_distance("R8,U5,L5,D3", "U7,R6,D4,L4"), Ok(6));
    }

    #[test]
    fn test_minimum_distance_case_2() {
        assert_eq!(
            find_minimum_distance(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83"
            ),
            Ok(159)
        );
    }

    #[test]
    fn test_minimum_distance_case_3() {
        assert_eq!(
            find_minimum_distance(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ),
            Ok(135)
        );
    }

    #[test]
    fn test_minimum_combined_steps_case_1() {
        assert_eq!(
            find_minimum_combined_steps("R8,U5,L5,D3", "U7,R6,D4,L4"),
            Ok(30)
        );
    }

    #[test]
    fn test_minimum_combined_steps_case_2() {
        assert_eq!(
            find_minimum_combined_steps(
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83"
            ),
            Ok(610)
        );
    }

    #[test]
    fn test_minimum_combined_steps_case_3() {
        assert_eq!(
            find_minimum_combined_steps(
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"
            ),
            Ok(410)
        );
    }
}
