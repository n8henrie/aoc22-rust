#![warn(clippy::pedantic)]

use aoc::{err, Error, Result};
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

    fn get_tree_mut(&mut self, (x, y): (usize, usize)) -> Result<&mut Tree> {
        self.0
            .get_mut(y)
            .and_then(|row| row.get_mut(x))
            .ok_or_else(|| err!("no tree at ({}, {})", x, y))
    }

    fn get_distances(&mut self) -> Result<()> {
        let (x_dim, y_dim) = self.dims()?;

        for (x, y) in (0..x_dim).flat_map(|x| (0..y_dim).map(move |y| (x, y))) {
            let height = self.get_tree_mut((x, y))?.height;

            if x == 0 {
                self.get_tree_mut((x, y))?.views.left = Some(0);
            } else {
                for dx in (0..x).rev() {
                    let left_tree = self.get_tree_mut((dx, y))?;
                    if left_tree.height >= height {
                        let diff = x - dx;
                        self.get_tree_mut((x, y))?.views.left = Some(diff);
                        break;
                    }
                }
                if self.get_tree_mut((x, y))?.views.left.is_none() {
                    self.get_tree_mut((x, y))?.views.left = Some(x);
                }
            }

            if x == (x_dim - 1) {
                self.get_tree_mut((x, y))?.views.right = Some(0);
            } else {
                for dx in (x + 1)..x_dim {
                    let right_tree = self.get_tree_mut((dx, y))?;
                    if right_tree.height >= height {
                        let diff = dx - x;
                        self.get_tree_mut((x, y))?.views.right = Some(diff);
                        break;
                    }
                }
                if self.get_tree_mut((x, y))?.views.right.is_none() {
                    self.get_tree_mut((x, y))?.views.right = Some(x_dim - x - 1);
                }
            }

            if y == 0 {
                self.get_tree_mut((x, y))?.views.up = Some(0);
            } else {
                for dy in (0..y).rev() {
                    let up_tree = self.get_tree_mut((x, dy))?;
                    if up_tree.height >= height {
                        let diff = y - dy;
                        self.get_tree_mut((x, y))?.views.up = Some(diff);
                        break;
                    }
                }
                if self.get_tree_mut((x, y))?.views.up.is_none() {
                    self.get_tree_mut((x, y))?.views.up = Some(y);
                }
            }

            if y == (y_dim - 1) {
                self.get_tree_mut((x, y))?.views.down = Some(0);
            } else {
                for dy in (y + 1)..y_dim {
                    let down_tree = self.get_tree_mut((x, dy))?;
                    if down_tree.height >= height {
                        let diff = dy - y;
                        self.get_tree_mut((x, y))?.views.down = Some(diff);
                        break;
                    }
                }
                if self.get_tree_mut((x, y))?.views.down.is_none() {
                    self.get_tree_mut((x, y))?.views.down = Some(y_dim - y - 1);
                }
            }
        }
        Ok(())
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

macro_rules! get_score {
    ($obj:expr, $($dir:ident),+) => {{
        [
            $(
                $obj.views.$dir.ok_or_else(|| $crate::err!("missing score"))
            ),*
        ].into_iter().collect::<$crate::Result<Vec<_>>>().map(|r| r.into_iter().map(|v| v as u32).product())
    }}
}

fn part2(forest: &mut Forest) -> Result<u32> {
    forest.get_distances()?;
    forest
        .as_grid()
        .map(|(_, tree)| get_score!(tree, left, right, up, down))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .ok_or_else(|| err!("no max found"))
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
    let mut forest: Forest = INPUT.parse()?;
    println!("day 08 part 1: {}", part1(&forest)?);
    println!("day 08 part 2: {}", part2(&mut forest)?);
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
    fn test_get_tree_mut() {
        let mut parsed: Forest = EXAMPLE_INPUT.parse().unwrap();
        assert_eq!(parsed.get_tree_mut((0, 0)).unwrap().height, 3);
        assert_eq!(parsed.get_tree_mut((0, 1)).unwrap().height, 2);
        assert_eq!(parsed.get_tree_mut((1, 1)).unwrap().height, 5);
        assert_eq!(parsed.get_tree_mut((4, 4)).unwrap().height, 0);
    }

    #[test]
    fn test_part2() {
        let mut forest: Forest = EXAMPLE_INPUT.parse().unwrap();
        forest.get_distances().unwrap();

        let tree = &forest.0[3][2];
        assert_eq!(tree.height, 5);

        let views = &tree.views;
        assert_eq!(views.up.unwrap(), 2);
        assert_eq!(views.left.unwrap(), 2);
        assert_eq!(views.down.unwrap(), 1);
        assert_eq!(views.right.unwrap(), 2);

        forest = EXAMPLE_INPUT.parse().unwrap();
        assert_eq!(part2(&mut forest).unwrap(), 8);
    }

    //     In the example above, consider the middle `5` in the second row:

    // ```
    // 30373
    // 25512
    // 65332
    // 33549
    // 35390

    // ```

    // * Looking up, its view is not blocked; it can see `*1*` tree (of height `3`).
    // * Looking left, its view is blocked immediately; it can see only `*1*` tree (of height `5`, right next to it).
    // * Looking right, its view is not blocked; it can see `*2*` trees.
    // * Looking down, its view is blocked eventually; it can see `*2*` trees (one of height `3`, then the tree of height `5` that blocks its view).
}
