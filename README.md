# aoc22-rust [![.github/workflows/ci.yml](https://github.com/n8henrie/aoc22-rust/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/aoc22-rust/actions/workflows/ci.yml)

I had a blast doing [Advent of Code 2018 in Rust][0], though I barely finished
in time for AoC 2019, and I was able to complete a handfull of [AoC
2021](https://github.com/n8henrie/aoc21-rust) as well. For 2022 I'm going to
try not to spend *too* much time on AoC, but I'll go through a few of them at
least.

My goals (mostly the same as 2021):

1. Have fun.
2. Hopefully see how much I've improved in the last couple years.
3. Look for opportunities to practice my many weak spots and new features I've
   been hoping to try out:
    - Generics
    - Const generics
    - Declarative macros (probably not procedural)
    - Parallelism / Rayon
    - Documenting my crates
    - Workspaces
    - Async? Doubt there is much opportunity in AoC

Anything I've cared to document is at <https://n8henrie.com/aoc22-rust/aoc/>.

I may also try a few problems in Go, Swift, or maybe something else entirely.
Time will tell.

## Other AoC '22 in Rust repos:

I'm going to split these into separate sections for beginners and
non-beginners, as learners like myself might want to compare and contrast
approaches.

You might also consider exploring a few using the GitHub API:

```console
$ gh api \
    -X GET search/repositories \
    --paginate \
    -f q='language:rust "advent of code"' \
    -f sort=stars \
    --jq '.items[] | .html_url'
https://github.com/warycat/rustgym
https://github.com/BurntSushi/advent-of-code
https://github.com/fspoettel/advent-of-code-rust
https://github.com/timvisee/advent-of-code-2021
https://github.com/scarvalhojr/aoc-cli
https://github.com/timvisee/advent-of-code-2020
https://github.com/simonw/advent-of-code-2022-in-rust
https://github.com/timvisee/advent-of-code-2022
https://github.com/aldanor/aoc-2021
https://github.com/AxlLind/AdventOfCode2021
...
```

### Intermediate / Advanced

- TBD

### Beginner level

These are repos from users that have self-identified as beging Rust novices /
learners / beginners. If you're on this list and take offense or think you
belong above, please let me know in an issue!

- <https://github.com/acehinnnqru/aoc-2022>
- <https://github.com/mario-hess/rust_advent_of_code_2022>

[0]: https://github.com/n8henrie/advent2018-rust
[1]: https://github.com/n8henrie/aoc21-rust
