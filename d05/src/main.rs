#![warn(clippy::pedantic)]
use std::{result, str::FromStr};

use aoc::{err, Error, Result};

const INPUT: &str = include_str!("../input.txt");

#[derive(PartialEq, Clone, Debug)]
struct Ship(Vec<Stack>);

#[derive(PartialEq, Clone, Debug)]
struct Stack(Vec<Crate>);

#[derive(PartialEq, Debug)]
struct Instruction {
    mv: usize,
    from: usize,
    to: usize,
}

type Crate = char;

impl FromStr for Ship {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut lines: Vec<_> = s.lines().filter(|line| !line.trim().is_empty()).collect();

        let Some(last_line) = lines.pop() else {
            return Err(err!("no last line: {:?}", lines));
        };

        let stack_count: usize = last_line
            .split_whitespace()
            .last()
            .and_then(|word| word.parse().ok())
            .ok_or_else(|| err!("couldn't parse stack count"))?;
        let mut stacks = vec![Stack(Vec::new()); stack_count];

        while let Some(line) = lines.pop() {
            let chars: Vec<char> = line.chars().collect();
            for (idx, chunk) in chars.chunks(4).enumerate() {
                match *chunk
                    .iter()
                    .filter(|c| c.is_alphabetic())
                    .collect::<Vec<_>>()
                    .as_slice()
                {
                    [c] => stacks[idx].0.push(*c),
                    [] => continue,
                    _ => return Err(err!("could not parse chunk: {:?}", chunk)),
                }
            }
        }
        Ok(Ship(stacks))
    }
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        if let ["move", mv, "from", from, "to", to] =
            *s.split_whitespace().collect::<Vec<_>>().as_slice()
        {
            let (mv, from, to) = (mv.parse()?, from.parse()?, to.parse()?);
            Ok(Self { mv, from, to })
        } else {
            Err(err!("unable to parse line as instruction: {}", s))
        }
    }
}

// RNZLFZSJH
fn part1(ship: &mut Ship, instructions: &[Instruction]) -> Result<String> {
    for Instruction { mv, to, from } in instructions {
        // Account for zero indexing
        let (to, from) = (to - 1, from - 1);
        for _ in 0..*mv {
            let Some(crate_) = ship.0[from].0.pop() else {
            return Err(err!("empty stack!"));
            };
            ship.0[to].0.push(crate_);
        }
    }

    Ok(ship
        .0
        .iter()
        .map(|stack| *stack.0.last().unwrap())
        .collect())
}

// CNSFCGJSM
fn part2(ship: &mut Ship, instructions: &[Instruction]) -> Result<String> {
    for Instruction { mv, to, from } in instructions {
        // Account for zero indexing
        let (to, from) = (to - 1, from - 1);
        let len = ship.0[from].0.len();

        if ship.0[from].0.get(len - mv..).is_none() {
            return Err(err!("not enough crates in {} (tried to get {})", from, mv));
        }
        let crates: Vec<_> = ship.0[from].0.drain(len - mv..).collect();
        ship.0[to].0.extend(crates);
    }

    Ok(ship
        .0
        .iter()
        .map(|stack| *stack.0.last().unwrap())
        .collect())
}

fn parse_input(input: &str) -> Result<(Ship, Vec<Instruction>)> {
    let mut splitter = input.split("\n\n");
    let (Some(ship_str), Some(instructions_str)) = (splitter.by_ref().next(), splitter.next()) else {
        return Err(err!("not enough parts to input"));
    };
    Ok((
        ship_str.parse()?,
        instructions_str
            .lines()
            .map(str::parse)
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn main() -> Result<()> {
    let (mut ship, instructions) = parse_input(INPUT)?;
    println!(
        "day 05 part 1: {}",
        part1(&mut ship.clone(), &instructions)?
    );
    println!("day 05 part 2: {}", part2(&mut ship, &instructions)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = r#"
    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
"#;

    #[test]
    fn test_parse_instruction() {
        let input = "move 1 from 2 to 1";
        let parsed: Instruction = input.parse().unwrap();
        let expected = Instruction {
            mv: 1,
            from: 2,
            to: 1,
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_ship() {
        let input = r#"
    [D]
[N] [C]
[Z] [M] [P]
 1   2   3
"#;

        let parsed: Ship = input.parse().unwrap();
        let expected = Ship(vec![
            Stack(vec!['Z', 'N']),
            Stack(vec!['M', 'C', 'D']),
            Stack(vec!['P']),
        ]);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_part1() {
        let (mut ship, instructions) = parse_input(EXAMPLE_INPUT).unwrap();
        assert_eq!(part1(&mut ship, &instructions).unwrap(), "CMZ");
    }

    #[test]
    fn test_part2() {
        let (mut ship, instructions) = parse_input(EXAMPLE_INPUT).unwrap();
        assert_eq!(part2(&mut ship, &instructions).unwrap(), "MCD");
    }
}
