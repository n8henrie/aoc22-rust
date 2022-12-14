#![warn(clippy::pedantic)]
use aoc::{err, Error, Result};

use std::{collections::HashSet, str::FromStr};

const INPUT: &str = include_str!("../input.txt");

type ItemType = char;
type ItemSet = HashSet<ItemType>;

#[derive(PartialEq, Debug)]
struct RuckSack(ItemSet, ItemSet);

impl RuckSack {
    fn in_common(&self) -> impl Iterator<Item = &ItemType> {
        self.0.intersection(&self.1)
    }

    fn score(&self) -> Result<u32> {
        self.in_common()
            .map(|&c| match c {
                'a'..='z' => Ok(c as u32 - 96),
                'A'..='Z' => Ok(c as u32 - 38),
                _ => Err(err!("unscorable character: {}", c)),
            })
            .sum()
    }
}

impl FromStr for RuckSack {
    type Err = Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(err!("empty input"));
        }
        let half = input.len() / 2;

        let mut iter = input.chars();
        Ok(RuckSack(
            iter.by_ref().take(half).collect(),
            iter.take(half).collect(),
        ))
    }
}

// 7878
fn part1(input: &str) -> Result<u32> {
    input
        .lines()
        .map(|line| {
            let r: RuckSack = line.parse()?;
            r.score()
        })
        .sum()
}

fn part2(input: &str) -> Result<u32> {
    let itemsets: Vec<ItemSet> =
        input.lines().map(|line| line.chars().collect()).collect();
    let badges = itemsets
        .chunks_exact(3)
        .map(|chunk| {
            let badge_candidates = chunk
                .iter()
                .cloned()
                .reduce(|acc, itemset| {
                    acc.intersection(&itemset).copied().collect()
                })
                .ok_or_else(|| err!("reduce was empty"))?;
            let len = badge_candidates.len();
            if len != 1 {
                return Err(err!("expected 1 badge candidate, found {}", len));
            };
            Ok(badge_candidates.into_iter().next().unwrap())
        })
        .collect::<Result<Vec<_>>>()?;

    badges
        .iter()
        .map(|&c| match c {
            'a'..='z' => Ok(c as u32 - 96),
            'A'..='Z' => Ok(c as u32 - 38),
            _ => Err(err!("unscorable character: {}", c)),
        })
        .sum()
}

fn main() -> Result<()> {
    println!("day 03 part 1: {}", part1(INPUT)?);
    println!("day 03 part 2: {}", part2(INPUT)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = "\
vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw
";

    #[test]
    fn test_make_rucksack() {
        let r: RuckSack =
            EXAMPLE_INPUT.lines().next().unwrap().parse().unwrap();
        let expected = RuckSack(
            "vJrwpWtwJgWr".chars().collect(),
            "hcsFMMfFFhFp".chars().collect(),
        );
        assert_eq!(r, expected);
    }

    #[test]
    fn test_score_ruckscack() {
        let r = RuckSack(
            "vJrwpWtwJgWr".chars().collect(),
            "hcsFMMfFFhFp".chars().collect(),
        );
        let common: ItemSet = r.in_common().copied().collect();
        assert_eq!(common, ItemSet::from(['p']));
        assert_eq!(r.score().unwrap(), 16);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE_INPUT).unwrap(), 157);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE_INPUT).unwrap(), 70);
    }
}
