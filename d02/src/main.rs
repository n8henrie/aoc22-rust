#![warn(clippy::pedantic)]
use aoc::{err, parse_input, Error, Result};
use std::result;
use std::str::FromStr;

const INPUT: &str = include_str!("../input.txt");

#[derive(Clone, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    fn score(&self) -> u32 {
        use Move::{Paper, Rock, Scissors};
        match self {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        }
    }
}

#[derive(PartialEq)]
enum Outcome {
    Win,
    Loss,
    Tie,
}

impl Outcome {
    fn score(&self) -> u32 {
        match self {
            Outcome::Win => 6,
            Outcome::Loss => 0,
            Outcome::Tie => 3,
        }
    }
}

struct Game(Move, Move);

impl Game {
    fn score(&self) -> u32 {
        self.1.score() + self.outcome().score()
    }

    fn outcome(&self) -> Outcome {
        use Move::{Paper, Rock, Scissors};
        use Outcome::{Loss, Tie, Win};
        match self {
            Game(a, b) if a == b => Tie,

            Game(Rock, Paper)
            | Game(Paper, Scissors)
            | Game(Scissors, Rock) => Win,

            Game(Paper, Rock)
            | Game(Scissors, Paper)
            | Game(Rock, Scissors) => Loss,

            _ => unreachable!("logic error"),
        }
    }
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let v = s.split_whitespace().collect::<Vec<_>>();
        let [lhs, rhs, ..] = v.as_slice() else {
            Err(err!("not enough items in line"))?
        };
        let (lhs, rhs) = (lhs.parse()?, rhs.parse()?);
        Ok(Self(lhs, rhs))
    }
}

impl FromStr for Move {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        use Move::{Paper, Rock, Scissors};
        Ok(match s {
            "A" | "X" => Rock,
            "B" | "Y" => Paper,
            "C" | "Z" => Scissors,
            x => return Err(err!("Unrecognized move: {}", x)),
        })
    }
}

// 13675
fn part1(parsed: &[Game]) -> u32 {
    parsed.iter().map(Game::score).sum()
}

// 14184
fn part2(parsed: &[Game]) -> Result<u32> {
    parsed
        .iter()
        .map(|game| {
            use Move::{Paper, Rock, Scissors};
            use Outcome::{Loss, Tie, Win};
            let outcome = match game.1 {
                Rock => Loss,
                Paper => Tie,
                Scissors => Win,
            };

            let Some(r#move) = [Rock, Paper, Scissors].into_iter().find(|r#move| {
                let game = Game(game.0.clone(), r#move.clone());
               game.outcome() == outcome
            }) else { return Err(err!("no suitable move found"))};
            Ok(Game(game.0.clone(), r#move).score())
        })
        .sum()
}

fn main() -> Result<()> {
    let parsed = parse_input!(INPUT, Game)?;
    println!("day 02 part 1: {}", part1(&parsed));
    println!("day 02 part 2: {}", part2(&parsed)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = "\
A Y
B X
C Z
";

    #[test]
    fn test_part1() {
        let parsed: Vec<Game> = parse_input!(EXAMPLE_INPUT, Game).unwrap();
        assert_eq!(part1(&parsed), 15);
    }

    #[test]
    fn test_part2() {
        let parsed: Vec<Game> = parse_input!(EXAMPLE_INPUT, Game).unwrap();
        assert_eq!(part2(&parsed).unwrap(), 12);
    }
}
