#![warn(clippy::pedantic)]
use aoc::{err, localpath, parse_input, Error, Result};

use std::result;
use std::{collections::HashSet, str::FromStr};

#[derive(PartialEq, Debug)]
struct Section(HashSet<usize>);

#[derive(PartialEq, Debug)]
struct ElfPair(Section, Section);

impl ElfPair {
    fn duplicated_effort(&self) -> bool {
        self.0 .0.is_subset(&self.1 .0) || self.1 .0.is_subset(&self.0 .0)
    }

    fn partial_overlaps(&self) -> bool {
        !self.0 .0.is_disjoint(&self.1 .0)
    }
}

fn part1(elfpairs: &[ElfPair]) -> usize {
    elfpairs.iter().filter(|ep| ep.duplicated_effort()).count()
}

fn part2(elfpairs: &[ElfPair]) -> usize {
    elfpairs.iter().filter(|ep| ep.partial_overlaps()).count()
}

impl FromStr for ElfPair {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut iter = s.split(',').map(FromStr::from_str);
        let (Some(Ok(first)), Some(Ok(second))) =
            (iter.by_ref().next(), iter.next()) else {
            return Err(err!("unable to parse as elfpair: {}", s));
            };
        Ok(Self(first, second))
    }
}

impl FromStr for Section {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut iter = s.split('-').map(str::parse);
        let (Some(Ok(start)), Some(Ok(end))) = (iter.by_ref().next(), iter.next()) else {
            return Err(err!("not enough values for section, found {}", s));
        };
        Ok(Self((start..=end).collect()))
    }
}

fn main() -> Result<()> {
    let pairs = parse_input!(localpath!("input.txt"), ElfPair)?;
    println!("day 04 part 1: {}", part1(&pairs));
    println!("day 04 part 2: {}", part2(&pairs));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = "\
2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8
";

    #[test]
    fn test_parse_section() {
        let section: Section = "2-4".parse().unwrap();
        assert_eq!(section, Section([2_usize, 3, 4].into_iter().collect()));
    }

    #[test]
    fn test_parse_elfpair() {
        let elfpair: ElfPair = EXAMPLE_INPUT.lines().next().unwrap().parse().unwrap();
        let expected = ElfPair(
            Section([2_usize, 3, 4].into_iter().collect()),
            Section([6_usize, 7, 8].into_iter().collect()),
        );
        assert_eq!(elfpair, expected);
    }

    #[test]
    fn test_part1() {
        let pairs = parse_input!(EXAMPLE_INPUT, ElfPair).unwrap();
        assert_eq!(part1(&pairs), 2);
    }

    #[test]
    fn test_part2() {
        let pairs = parse_input!(EXAMPLE_INPUT, ElfPair).unwrap();
        assert_eq!(part2(&pairs), 4);
    }
}
