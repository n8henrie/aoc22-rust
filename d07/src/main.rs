// #![warn(clippy::pedantic)]
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

    fn add_child(base: &Rc<RefCell<Dir>>, item: &Rc<RefCell<Item>>) {
        match *item.as_ref().borrow_mut() {
            Item::Dir(ref mut d) => d.borrow_mut().parent = Some(Rc::downgrade(base)),
            Item::File(ref mut f) => f.borrow_mut().parent = Some(Rc::downgrade(base)),
        };
        base.borrow_mut().children.push(Rc::clone(item));
    }

    fn root() -> Rc<RefCell<Self>> {
        Dir::new("/", None)
    }

    fn get_parent(&self) -> Option<Weak<RefCell<Dir>>> {
        self.parent.clone()
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    // fn cd(from: &mut Rc<RefCell<Self>>, dir: impl AsRef<str>) {
    //     match dir.as_ref() {
    //         "/" => {
    //             while let Some(p) = from.borrow().get_parent() {
    //                 *from = p;
    //             }
    //         }
    //         _ => todo!(),
    //     }
    // }
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
    fn parent(&self) -> Option<Weak<RefCell<Dir>>>;
    fn name(&self) -> String;
    fn abspath(&self) -> Result<PathBuf> {
        let mut v = vec![self.name()];
        let mut parent = self.parent().as_ref().and_then(|p| p.upgrade());
        while let Some(p) = parent {
            v.push(p.borrow().name.clone());
            parent = p
                .as_ref()
                .borrow()
                .parent()
                .as_ref()
                .and_then(|p| p.upgrade());
        }
        Ok(v.into_iter().rev().collect())
    }
}

impl Child for Item {
    fn parent(&self) -> Option<Weak<RefCell<Dir>>> {
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
    fn parent(&self) -> Option<Weak<RefCell<Dir>>> {
        self.parent.clone()
    }
    fn name(&self) -> String {
        self.name.to_string()
    }
}

impl Child for File {
    fn parent(&self) -> Option<Weak<RefCell<Dir>>> {
        self.parent.clone()
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

impl Item {
    fn get_parent(&self) -> Option<Rc<RefCell<Dir>>> {
        match *self {
            Item::Dir(ref d) => d.as_ref().borrow().parent.as_ref()?.upgrade(),
            Item::File(ref f) => f.as_ref().borrow().parent.as_ref()?.upgrade(),
        }
    }
}

fn parse_input(input: &str) -> Result<Item> {
    let mut cwd: Option<Rc<RefCell<Dir>>> = None;
    for line in input.lines() {
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["$", "cd", "/"] => {
                cwd = Some(Dir::root());
            }
            ["$", "cd", dir] => {
                let Some(cwd) = cwd.as_mut() else {
                return Err(err!("Attempt to cd while uninitialized"))
                };
                // Dir::cd(cwd, dir);
            }
            ["$", "ls"] => (),
            [num, name] if num.parse::<u32>().is_ok() => {
                let num = num.parse::<u32>().unwrap();
                let file = File::new(name, num);
            }
            ["dir", name] => todo!(),
            _ => return Err(err!("Unrecognized input: {}", line)),
        }
    }
    todo!()
}

fn part1(input: &str) -> Result<u32> {
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
    }

    #[test]
    fn test_parse_input() {
        const OUTPUT: &str = "\
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
        let parsed = parse_input(EXAMPLE_INPUT).unwrap();
    }

    #[test]
    fn test_part1() {
        assert!(false);
    }

    #[test]
    fn test_part2() {
        assert!(false);
    }
}
