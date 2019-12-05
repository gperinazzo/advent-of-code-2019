use std::collections::HashSet;

fn is_valid_puzzle_1(value: u32) -> bool {
    let mut set: HashSet<u32> = HashSet::with_capacity(6);
    let mut last_digit = 0;
    for i in (0..6).rev() {
        let digit = (value / 10_u32.pow(i)) % 10;
        if digit < last_digit {
            return false;
        }

        last_digit = digit;
        set.insert(digit);
    }
    set.len() < 6
}

fn is_valid_puzzle_2(value: u32) -> bool {
    let mut repetitions: [u8; 10] = [0; 10];
    let mut last_digit = 0;
    for i in (0..6).rev() {
        let digit = (value / 10_u32.pow(i)) % 10;
        if digit < last_digit {
            return false;
        }

        last_digit = digit;
        repetitions[last_digit as usize] += 1;
    }

    repetitions.iter().find(|value| **value == 2).is_some()
}

fn main() {
    let count = (172851..=675869)
        .filter(|value| is_valid_puzzle_1(*value))
        .count();
    println!("Puzzle 1 - {}", count);

    let count = (172851..=675869)
        .filter(|value| is_valid_puzzle_2(*value))
        .count();
    println!("Puzzle 2 - {}", count);
}

#[cfg(test)]
mod test {
    use super::{is_valid_puzzle_1, is_valid_puzzle_2};

    #[test]
    fn test_valid_puzzle_1() {
        assert_eq!(is_valid_puzzle_1(111111), true);
        assert_eq!(is_valid_puzzle_1(123456), false);
        assert_eq!(is_valid_puzzle_1(122345), true);
    }

    #[test]
    fn test_valid_puzzle_2() {
        assert_eq!(is_valid_puzzle_2(111111), false);
        assert_eq!(is_valid_puzzle_2(123456), false);
        assert_eq!(is_valid_puzzle_2(122345), true);
        assert_eq!(is_valid_puzzle_2(122245), false);
        assert_eq!(is_valid_puzzle_2(123444), false);
        assert_eq!(is_valid_puzzle_2(111122), true);
    }
}
