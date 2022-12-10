#![warn(clippy::pedantic)]
use std::collections::HashMap;

type ElfMap = HashMap<usize, u32>;

static INPUT: &str = include_str!("../input.txt");

fn parse_input(input: &str) -> anyhow::Result<ElfMap> {
    input.split("\n\n").enumerate().fold(
        Ok(HashMap::new()),
        |hm, (idx, elem)| {
            let mut hm = hm?;
            hm.insert(
                idx,
                elem.lines()
                    .fold(Ok::<_, anyhow::Error>(0), |acc, line| {
                        Ok(acc? + line.parse::<u32>()?)
                    })?,
            );
            Ok(hm)
        },
    )
}

// 70764
fn part1(parsed: &ElfMap) -> anyhow::Result<u32> {
    parsed
        .values()
        .copied()
        .max()
        .ok_or_else(|| anyhow::anyhow!("No max found"))
}

// 203905
fn part2(parsed: &ElfMap) -> u32 {
    let mut vals: Vec<_> = parsed.values().copied().collect();
    vals.sort_unstable();
    vals.iter().rev().take(3).sum()
}

fn main() -> anyhow::Result<()> {
    // let input = parse_input!(localpath!("input.txt"))?;
    // let input = std::fs::read_to_string("input.txt")?;
    let parsed = parse_input(INPUT)?;
    println!("day 01 part 1: {}", part1(&parsed)?);
    println!("day 02 part 2: {}", part2(&parsed));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = "\
1000
2000
3000

4000

5000
6000

7000
8000
9000

10000";

    #[test]
    fn test_parse() {
        let parsed = parse_input(EXAMPLE_INPUT).unwrap();

        assert_eq!(parsed[&0], 6000);
        assert_eq!(parsed[&1], 4000);
        assert_eq!(parsed[&2], 11000);
        assert_eq!(parsed[&3], 24000);
        assert_eq!(parsed[&4], 10000);
    }

    #[test]
    fn test_part1() {
        assert_eq!(
            part1(&parse_input(EXAMPLE_INPUT).unwrap()).unwrap(),
            24000
        );
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(&parse_input(EXAMPLE_INPUT).unwrap()), 45000);
    }
}
