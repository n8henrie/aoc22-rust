pub use anyhow;

/// Return a path to the input, starting at `CARGO_MANIFEST_DIR` of the
/// *caller*, in a way that compiles the result into the final binary.
///
// Didn't work:
//   - using a function with `env!` -- uses the `CARGO_MANIFEST_DIR` of this
//   crate instead of the caller
//   - use runtime env with `std::env` -- doesn't compile into the binary so
//   fails if not run via `cargo run`
#[macro_export]
macro_rules! localpath {
    ($tt:tt) => {{
        use ::std::path::Path;
        let envvar = env!("CARGO_MANIFEST_DIR");
        let basedir = Path::new(&envvar);
        basedir.join($tt)
    }};
}

/// Parse input into a vec of specified type, or default to `Vec<String>`.
/// Test out a link to [parse_input].
/// ```rust
/// use aoc::parse_input;
/// assert_eq!(parse_input!("42\n24", u32).unwrap(), vec![42_u32, 24]);
/// assert_eq!(parse_input!("42").unwrap(), vec![String::from("42")]);
/// ```
#[macro_export]
macro_rules! parse_input {
    ($path:expr) => {
        parse_input!($path, String)
    };
    ($path:expr, $ty:ty) => {{
        use ::std::boxed::Box;
        use ::std::fs::File;
        use ::std::io::Read;
        use ::std::io::{BufRead, BufReader};
        use ::std::path::PathBuf;
        use $crate::files::anyhow;

        let path = PathBuf::from($path);
        let file = File::open(&path);

        let input: Box<dyn Read> = if let Ok(f) = file {
            Box::new(f)
        } else {
            Box::new(path.to_str().unwrap().as_bytes())
        };
        BufReader::new(input)
            .lines()
            .map(|bufline| {
                anyhow::Context::context(
                    bufline,
                    "error iterating over bufreader",
                )
                .and_then(|line| {
                    anyhow::Context::context(
                        line.parse::<$ty>().map_err(|e| anyhow::anyhow!(e)),
                        "Unable to parse as type",
                    )
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    static U32_TEST: &str = "123\n456";

    #[test]
    fn test_localpath() {
        let path = localpath!("foo.txt");
        assert!(path.ends_with("aoc22-rust/aoc/foo.txt"));
    }

    #[test]
    fn test_read_to_u32_from_file() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(tmpfile, "{}", U32_TEST).unwrap();
        let expected = vec![123_u32, 456];
        let result = parse_input!(tmpfile.path(), u32);
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_read_to_u32_from_str() {
        let result = parse_input!(U32_TEST, u32);
        let expected = vec![123_u32, 456];
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_read_to_string() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(tmpfile, "123\n456").unwrap();
        let expected: Vec<String> =
            ["123", "456"].into_iter().map(Into::into).collect();
        let result = parse_input!(tmpfile.path()).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_read_to_custom_type() {
        #[derive(Debug, PartialEq)]
        struct Point {
            x: u32,
            y: u32,
        }

        impl std::str::FromStr for Point {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut splitter = s.split_whitespace();
                match (splitter.next(), splitter.next()) {
                    (Some(x), Some(y)) => Ok(Point {
                        x: x.parse()?,
                        y: y.parse()?,
                    }),
                    _ => anyhow::bail!("Unable to parse"),
                }
            }
        }

        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write!(tmpfile, "1 2\n3 4").unwrap();
        let expected = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
        let result = parse_input!(tmpfile.path(), Point).unwrap();
        assert_eq!(expected, result);
    }
}
