#![warn(clippy::pedantic)]
use aoc::{err, localpath, parse_input, Error, Result};

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

const INPUT: &str = include_str!("../input.txt");

#[derive(Clone, Debug, Default)]
struct Dir {
    name: String,
    parent: Option<Weak<RefCell<Self>>>,
    children: Vec<Rc<RefCell<Item>>>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self.abspath(), other.abspath()) {
            (Ok(first), Ok(second)) => first == second,
            _ => false,
        }
    }
}

impl Dir {
    fn new(name: impl AsRef<str>, parent: Option<Weak<RefCell<Dir>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            name: name.as_ref().to_string(),
            parent,
            children: Vec::new(),
        }))
    }

    fn add_child(base: &Rc<RefCell<Self>>, item: &Rc<RefCell<Item>>) {
        match *item.borrow_mut() {
            Item::Dir(ref d) => {
                d.borrow_mut().parent = Some(Rc::downgrade(base));
            }
            Item::File(ref f) => f.borrow_mut().parent = Some(Rc::downgrade(base)),
        };
        base.borrow_mut().children.push(Rc::clone(item));
    }

    fn root() -> Rc<RefCell<Self>> {
        Dir::new("/", None)
    }

    fn cd(from: &mut Rc<RefCell<Self>>, dir: impl AsRef<str>) -> Result<()> {
        match dir.as_ref() {
            "/" => {
                let mut parent = from.borrow().parent.as_ref().and_then(|p| p.upgrade());
                while let Some(p) = parent {
                    parent = p.borrow().parent.as_ref().and_then(|p| p.upgrade());
                    if p.borrow().name == "/" {
                        return Ok(());
                    }
                }
                return Err(err!("Was not anchored to root"));
            }
            ".." => {
                if let Some(parent) = from.clone().borrow().parent() {
                    *from = Rc::clone(&parent);
                    Ok(())
                } else {
                    Err(err!(
                        "Tried to `cd ..` from `{}`, which has no parent directory",
                        from.borrow().name
                    ))
                }
            }
            dest => {
                for child in from.clone().borrow().children.iter() {
                    if let Item::Dir(ref d) = *child.borrow() {
                        if d.borrow().name == dest {
                            *from = Rc::clone(d);
                            return Ok(());
                        }
                    }
                }
                Err(err!("tried to cd to subdir `{}` that doesn't exist", dest))
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct File {
    parent: Option<Weak<RefCell<Dir>>>,
    name: String,
    size: u32,
}

impl File {
    fn new(name: impl AsRef<str>, size: u32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            name: name.as_ref().to_string(),
            size,
            parent: None,
        }))
    }
}

trait Sizeable {
    fn size(&self) -> u32;
}

impl Sizeable for File {
    fn size(&self) -> u32 {
        self.size
    }
}

impl Sizeable for Dir {
    fn size(&self) -> u32 {
        self.children.iter().map(|c| c.borrow().size()).sum()
    }
}

impl Sizeable for Item {
    fn size(&self) -> u32 {
        match self {
            Item::File(f) => f.borrow().size(),
            Item::Dir(d) => d.borrow().size(),
        }
    }
}

trait Child {
    fn parent(&self) -> Option<Rc<RefCell<Dir>>>;
    fn name(&self) -> String;
    fn abspath(&self) -> Result<PathBuf> {
        let mut v = vec![self.name()];
        let mut parent = self.parent();
        while let Some(ref p) = parent {
            let name = p.borrow().name().to_string();
            v.push(name);
            parent = p.clone().borrow().parent();
        }
        Ok(v.into_iter().rev().collect())
    }
}

impl Child for Item {
    fn parent(&self) -> Option<Rc<RefCell<Dir>>> {
        match self {
            Item::Dir(d) => <Dir as Child>::parent(&d.borrow()),
            Item::File(f) => <File as Child>::parent(&f.borrow()),
        }
    }
    fn name(&self) -> String {
        match self {
            Item::Dir(d) => <Dir as Child>::name(&d.borrow()),
            Item::File(f) => <File as Child>::name(&f.borrow()),
        }
    }
}

impl Child for Dir {
    fn parent(&self) -> Option<Rc<RefCell<Dir>>> {
        self.parent.as_ref()?.upgrade().clone()
    }
    fn name(&self) -> String {
        self.name.to_string()
    }
}

impl Child for File {
    fn parent(&self) -> Option<Rc<RefCell<Dir>>> {
        self.parent.as_ref()?.upgrade()
    }
    fn name(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Clone, Debug)]
enum Item {
    Dir(Rc<RefCell<Dir>>),
    File(Rc<RefCell<File>>),
}

fn parse_input(input: &str) -> Result<Rc<RefCell<Dir>>> {
    let mut root = None;
    let mut cwd: Option<Rc<RefCell<Dir>>> = None;
    for line in input.lines() {
        dbg!(&line);
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["$", "cd", "/"] => {
                root = Some(Dir::root());
                cwd = root.clone();
            }
            ["$", "cd", dir] => {
                let Some(ref mut cwd) = cwd else {
                    return Err(err!("Attempt to cd while uninitialized"));
                };
                Dir::cd(cwd, dir)?;
            }
            ["$", "ls"] => (),
            ["dir", name] => {
                let Some(ref mut cwd) = cwd else {
                    return Err(err!("Attempt to add a dir with no cwd"));
                };
                let dir = Rc::new(RefCell::new(Item::Dir(Dir::new(name, None))));
                Dir::add_child(cwd, &dir);
            }
            [num, name] if num.parse::<u32>().is_ok() => {
                let Some(ref mut cwd) = cwd else {
                    return Err(err!("Attempt to add a file with no cwd"));
                };
                let num = num.parse::<u32>().unwrap();
                let file = Rc::new(RefCell::new(Item::File(File::new(name, num))));
                Dir::add_child(cwd, &file);
            }
            _ => return Err(err!("Unrecognized input: {}", line)),
        }
    }
    root.ok_or_else(|| err!("input not parsed"))
}

impl Item {
    fn iter(&self) -> ItemIterator {
        ItemIterator {
            stack: vec![Rc::new(RefCell::new(self.clone()))],
        }
    }
}

struct ItemIterator {
    stack: Vec<Rc<RefCell<Item>>>,
}

impl Iterator for ItemIterator {
    type Item = Rc<RefCell<Item>>;
    fn next(&mut self) -> Option<Self::Item> {
        let rc = self.stack.pop()?;
        match &*rc.clone().borrow() {
            Item::File(_) => Some(rc),
            Item::Dir(d) => {
                self.stack.extend(d.borrow().children.iter().rev().cloned());
                Some(rc)
            }
        }
    }
}

fn part1(input: &str, size_limit: u32) -> Result<u32> {
    let root = parse_input(input)?;

    todo!()
}

fn part2() -> Result<u32> {
    todo!()
}

fn main() -> Result<()> {
    // let input = parse_input!(localpath!("input.txt"))?;
    // println!("day 07 part 1: {}", part1(&input)?);
    // println!("day 07 part 2: {}", part2(&input)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EXAMPLE_INPUT: &str = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k
";

    #[test]
    fn test_node_construction() {
        let root = Dir::new("/", Default::default());
        let child_file = File::new("foo", 7);
        let child_dir = Dir::new("bar", Some(Rc::downgrade(&root)));

        Dir::add_child(
            &root,
            &Rc::new(RefCell::new(Item::File(child_file.clone()))),
        );
        Dir::add_child(&root, &Rc::new(RefCell::new(Item::Dir(child_dir))));

        let expected_child_file = File::new("foo", 7);
        let expected_child_dir = Dir::new("bar", None);
        let expected = Dir::new("/", None);
        expected_child_file.as_ref().borrow_mut().parent = Some(Rc::downgrade(&expected));
        expected_child_dir.as_ref().borrow_mut().parent = Some(Rc::downgrade(&expected));
        expected
            .borrow_mut()
            .children
            .push(Rc::new(RefCell::new(Item::File(
                expected_child_file.clone(),
            ))));
        expected
            .borrow_mut()
            .children
            .push(Rc::new(RefCell::new(Item::Dir(expected_child_dir))));

        assert_eq!(
            expected_child_file.borrow().abspath().unwrap(),
            child_file.borrow().abspath().unwrap()
        );
    }

    #[test]
    fn test_abspath() {
        let root = Dir::new("/", Default::default());
        let child_dir = Dir::new("bar", Some(Rc::downgrade(&root)));
        let child_file = Rc::new(RefCell::new(Item::File(File::new("foo", 7))));
        let subdir_file = Rc::new(RefCell::new(Item::File(File::new("baz", 14))));

        Dir::add_child(&child_dir, &subdir_file);
        Dir::add_child(&root, &child_file);

        let child_dir = Rc::new(RefCell::new(Item::Dir(child_dir)));
        Dir::add_child(&root, &child_dir);

        assert_eq!(
            child_file.borrow().abspath().unwrap(),
            PathBuf::from("/foo")
        );
        let binding = root.borrow();
        let Item::Dir(ref d) = *binding.children[1].borrow() else {
            panic!("second child should be a dir")
    };
        assert_eq!(d.borrow().abspath().unwrap(), PathBuf::from("/bar"));
    }

    #[test]
    fn test_size() {
        let root = Dir::new("/", Default::default());
        let child_dir = Dir::new("bar", Some(Rc::downgrade(&root)));
        let child_file = Rc::new(RefCell::new(Item::File(File::new("foo", 7))));
        let subdir_file = Rc::new(RefCell::new(Item::File(File::new("baz", 14))));

        Dir::add_child(&child_dir, &subdir_file);
        Dir::add_child(&root, &child_file);

        let child_dir = Rc::new(RefCell::new(Item::Dir(child_dir)));
        Dir::add_child(&root, &child_dir);

        assert_eq!(Item::Dir(root).size(), 21);
        assert_eq!(child_dir.borrow().size(), 14);
        assert_eq!(child_file.borrow().size(), 7);

        let root = parse_input(EXAMPLE_INPUT).unwrap();
        let cwd = &mut root.clone();
        // root, a, d, e
        assert_eq!(cwd.borrow().size(), 48381165);
        Dir::cd(cwd, "d");
        assert_eq!(cwd.borrow().size(), 24933642);
        Dir::cd(cwd, "..");
        Dir::cd(cwd, "a");
        assert_eq!(cwd.borrow().size(), 94853);
        Dir::cd(cwd, "e");
        assert_eq!(cwd.borrow().size(), 584);
    }

    #[test]
    fn test_parse_input() {
        const _OUTPUT: &str = "\
- / (dir)
  - a (dir)
    - e (dir)
      - i (file, size=584)
    - f (file, size=29116)
    - g (file, size=2557)
    - h.lst (file, size=62596)
  - b.txt (file, size=14848514)
  - c.dat (file, size=8504156)
  - d (dir)
    - j (file, size=4060174)
    - d.log (file, size=8033020)
    - d.ext (file, size=5626152)
    - k (file, size=7214296)
";
        parse_input(EXAMPLE_INPUT).unwrap();
    }

    #[test]
    fn test_add_child() {
        let root = Dir::new("/", Default::default());
        let child_dir = Dir::new("bar", None);
        let child_file = Rc::new(RefCell::new(Item::File(File::new("foo", 7))));
        let subdir_file = Rc::new(RefCell::new(Item::File(File::new("baz", 14))));
        assert!(root.borrow().parent().is_none());

        assert!(child_file.borrow().parent().is_none());
        Dir::add_child(&root, &child_file);
        assert!(child_file.borrow().parent().is_some());

        Dir::add_child(&child_dir, &subdir_file);

        let child_dir = Rc::new(RefCell::new(Item::Dir(child_dir)));
        assert!(child_dir.borrow().parent().is_none());
        Dir::add_child(&root, &child_dir);
        assert!(child_dir.borrow().parent().is_some());
    }

    #[test]
    fn test_cd() {
        let root = Dir::new("/", Default::default());
        let child_dir = Dir::new("bar", None);
        let child_file = Rc::new(RefCell::new(Item::File(File::new("foo", 7))));
        let subdir_file = Rc::new(RefCell::new(Item::File(File::new("baz", 14))));

        Dir::add_child(&root, &child_file);
        Dir::add_child(&child_dir, &subdir_file);

        let child_dir = Rc::new(RefCell::new(Item::Dir(child_dir)));
        Dir::add_child(&root, &child_dir);
        assert!(child_dir.borrow().parent().is_some());

        let mut cwd = root.clone();
        assert_eq!(cwd.borrow().name, "/");
        assert!(cwd.borrow().parent().is_none());

        assert!(child_dir.borrow().parent().is_some());
        Dir::cd(&mut cwd, "bar").unwrap();
        assert!(child_dir.borrow().parent().is_some());

        assert_eq!(cwd.borrow().name, "bar");
        assert!(cwd.borrow().parent().is_some());
        assert_eq!(cwd.borrow().abspath().unwrap().to_str().unwrap(), "/bar");

        assert!(Dir::cd(&mut cwd, "baz").is_err());
        assert_eq!(cwd.borrow().name, "bar");

        Dir::cd(&mut cwd, "..").unwrap();
        assert_eq!(cwd.borrow().name, "/");
    }

    // - / (dir)
    //   - a (dir)
    //     - e (dir)
    //       - i (file, size=584)
    //     - f (file, size=29116)
    //     - g (file, size=2557)
    //     - h.lst (file, size=62596)
    //   - b.txt (file, size=14848514)
    //   - c.dat (file, size=8504156)
    //   - d (dir)
    //     - j (file, size=4060174)
    //     - d.log (file, size=8033020)
    //     - d.ext (file, size=5626152)
    //     - k (file, size=7214296)
    #[test]
    fn test_item_iter() {
        let parsed = parse_input(EXAMPLE_INPUT).unwrap();
        let mut items = Item::iter(&Item::Dir(parsed));

        assert_eq!(items.next().unwrap().borrow().name(), "/");
        assert_eq!(items.next().unwrap().borrow().name(), "a");
        assert_eq!(items.next().unwrap().borrow().name(), "e");
        assert_eq!(items.next().unwrap().borrow().name(), "i");
        assert_eq!(items.next().unwrap().borrow().name(), "f");
        assert_eq!(items.next().unwrap().borrow().name(), "g");
        assert_eq!(items.next().unwrap().borrow().name(), "h.lst");
        assert_eq!(items.next().unwrap().borrow().name(), "b.txt");
        assert_eq!(items.next().unwrap().borrow().name(), "c.dat");
        assert_eq!(items.next().unwrap().borrow().name(), "d");
        assert_eq!(items.next().unwrap().borrow().name(), "j");
        assert_eq!(items.next().unwrap().borrow().name(), "d.log");
        assert_eq!(items.next().unwrap().borrow().name(), "d.ext");
        assert_eq!(items.next().unwrap().borrow().name(), "k");
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE_INPUT, 100000).unwrap(), 95437);
    }

    #[test]
    fn test_part2() {
        // assert!(false);
    }
}
