#![warn(clippy::pedantic)]

use aoc::{err, localpath, parse_input, Error, Result};
use std::str::FromStr;

const INPUT: &str = include_str!("../input.txt");

impl Forest {
    fn as_grid(&self) -> impl Iterator<Item = ((usize, usize), &Tree)> {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, tree)| ((x, y), tree)))
    }

    fn dims(&self) -> Result<(usize, usize)> {
        let y_dim = self.0.len();
        if y_dim == 0 {
            return Err(err!("empty forest"));
        };
        let x_dim = self.0[0].len();
        if x_dim == 0 {
            return Err(err!("empty forest"));
        };
        Ok((x_dim, y_dim))
    }
}

fn part1(forest: &Forest) -> Result<u32> {
    let (x_dim, y_dim) = forest.dims()?;

    Ok(forest.as_grid().fold(0, |acc, ((x, y), tree)| {
        if x == 0 || y == 0 || x == (x_dim - 1) || y == (y_dim - 1) {
            return acc + 1;
        }
        if (0..x).all(|x| forest.0[y][x].height < tree.height)
            || ((x + 1)..x_dim).all(|x| forest.0[y][x].height < tree.height)
            || (0..y).all(|y| forest.0[y][x].height < tree.height)
            || ((y + 1)..y_dim).all(|y| forest.0[y][x].height < tree.height)
        {
            acc + 1
        } else {
            acc
        }
    }))
}

fn part2(forest: &mut Forest) -> Result<u32> {
    let (x_dim, y_dim) = forest.dims()?;

    // Ok(forest.as_grid().fold(0, |acc, ((x, y), tree)| {}))
    todo!()
}

type Height = u8;

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct Views {
    left: Option<usize>,
    right: Option<usize>,
    up: Option<usize>,
    down: Option<usize>,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct Tree {
    height: Height,
    views: Views,
}

impl Tree {
    fn new(height: Height) -> Self {
        Self {
            height,
            ..Default::default()
        }
    }
}

impl FromStr for Tree {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Tree::new(s.parse::<Height>()?))
    }
}

impl FromStr for Forest {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(
            s.trim()
                .lines()
                .map(|line| {
                    line.chars()
                        .map(|c| c.to_string().parse::<Tree>())
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

#[derive(Debug)]
struct Forest(Vec<Vec<Tree>>);

fn main() -> Result<()> {
    let forest: Forest = INPUT.parse()?;
    // let input = parse_input!(localpath!("input.txt"), Forest)?;
    println!("day 08 part 1: {}", part1(&forest)?);
    // println!("day 08 part 2: {}", part2(&input)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = r#"
30373
25512
65332
33549
35390
"#;

    #[test]
    fn test_parse() {
        let parsed: Forest = EXAMPLE_INPUT.parse().unwrap();
        assert_eq!(parsed.0[0][0].height, 3);
        assert_eq!(parsed.0[1][0].height, 2);
        assert_eq!(parsed.0[1][1].height, 5);
        assert_eq!(parsed.0[4][4].height, 0);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(&EXAMPLE_INPUT.parse().unwrap()).unwrap(), 21);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(&mut EXAMPLE_INPUT.parse().unwrap()).unwrap(), 8);
    }
}
