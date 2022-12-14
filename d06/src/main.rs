#![warn(clippy::pedantic)]
use aoc::{err, Result};

use std::collections::HashSet;

const INPUT: &str = include_str!("../input.txt");

// 1804
fn scan_buffer(buffer: &str, window_size: usize) -> Result<usize> {
    let chars: Vec<_> = buffer.chars().collect();
    chars
        .windows(window_size)
        .enumerate()
        .find_map(|(idx, window)| {
            let hs: HashSet<&char> = window.iter().collect();
            if hs.len() == window_size {
                return Some(idx + window_size);
            };
            None
        })
        .ok_or_else(|| err!("No marker found!"))
}

// 1804
fn part1(buffer: &str) -> Result<usize> {
    scan_buffer(buffer, 4)
}

// 2508
fn part2(buffer: &str) -> Result<usize> {
    scan_buffer(buffer, 14)
}

fn main() -> Result<()> {
    println!("day 06 part 1: {}", part1(INPUT)?);
    println!("day 06 part 2: {}", part2(INPUT)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: [(&str, (usize, usize)); 5] = [
        ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", (7, 19)),
        ("bvwbjplbgvbhsrlpgdmjqwftvncz", (5, 23)),
        ("nppdvjthqldpwncqszvftbrmjlhg", (6, 23)),
        ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", (10, 29)),
        ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", (11, 26)),
    ];

    #[test]
    fn test_part1() {
        for (input, (expected, _)) in EXAMPLE_INPUT {
            assert_eq!(part1(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_part2() {
        for (input, (_, expected)) in EXAMPLE_INPUT {
            assert_eq!(part2(input).unwrap(), expected);
        }
    }
}
