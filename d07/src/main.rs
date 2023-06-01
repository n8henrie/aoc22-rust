#![cfg_attr(feature = "bench", feature(test))]
#![allow(dead_code)]
#![warn(clippy::pedantic)]
use aoc::{err, Result};

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

use std::ops::ControlFlow;

const INPUT: &str = include_str!("../input.txt");

#[derive(Debug)]
struct Arena(Vec<ArenaItem>);
type ArenaIndex = usize;

#[derive(Debug)]
enum ArenaItem {
    Dir(ArenaDir),
    File(ArenaFile),
}

impl ArenaItem {
    fn name(&self) -> &str {
        match self {
            ArenaItem::File(f) => &f.name,
            ArenaItem::Dir(d) => &d.name,
        }
    }

    fn set_parent(&mut self, parent_idx: ArenaIndex) {
        match self {
            ArenaItem::File(f) => f.parent = Some(parent_idx),
            ArenaItem::Dir(d) => d.parent = Some(parent_idx),
        }
    }
}

#[derive(Debug)]
struct ArenaDir {
    parent: Option<ArenaIndex>,
    children: Vec<ArenaIndex>,
    name: String,
}

impl ArenaDir {
    fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            parent: None,
            children: Vec::new(),
        }
    }
}

impl From<ArenaDir> for ArenaItem {
    fn from(dir: ArenaDir) -> Self {
        ArenaItem::Dir(dir)
    }
}

#[derive(Debug)]
struct ArenaFile {
    parent: Option<ArenaIndex>,
    name: String,
    size: u32,
}

impl ArenaFile {
    fn new(name: impl AsRef<str>, size: u32) -> Self {
        Self {
            name: name.as_ref().to_string(),
            parent: None,
            size,
        }
    }
}

impl From<ArenaFile> for ArenaItem {
    fn from(file: ArenaFile) -> Self {
        ArenaItem::File(file)
    }
}

impl Arena {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_item(&mut self, item: impl Into<ArenaItem>, parent: Option<ArenaIndex>) -> ArenaIndex {
        let idx = self.0.len();
        self.0.push(item.into());

        if let Some(parent_idx) = parent {
            self.0[idx].set_parent(parent_idx);

            if let ArenaItem::Dir(ref mut dir) = self.0[parent_idx] {
                dir.children.push(idx);
            };
        }
        idx
    }

    fn at(&self, idx: ArenaIndex) -> &ArenaItem {
        &self.0[idx]
    }

    fn dir_at(&self, idx: ArenaIndex) -> Result<&ArenaDir> {
        if let ArenaItem::Dir(dir) = self.at(idx) {
            Ok(dir)
        } else {
            Err(err!("ArenaItem at {idx} is not a dir"))
        }
    }

    fn iter_indices(&self) -> impl Iterator<Item = ArenaIndex> + '_ {
        let mut stack = vec![0];

        std::iter::from_fn(move || {
            let next = stack.pop()?;
            match self.at(next) {
                ArenaItem::Dir(dir) => {
                    stack.extend(dir.children.iter().rev().clone());
                    Some(next)
                }
                ArenaItem::File(_) => Some(next),
            }
        })
    }

    fn iter(&self) -> impl Iterator<Item = &ArenaItem> {
        self.iter_indices().map(|idx| self.at(idx))
    }

    fn size(&self, idx: ArenaIndex) -> u32 {
        match self.at(idx) {
            ArenaItem::Dir(d) => d.children.iter().map(|idx| self.size(*idx)).sum(),
            ArenaItem::File(f) => f.size,
        }
    }
}

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
                let mut parent = from.borrow().parent.as_ref().and_then(Weak::upgrade);
                while let Some(p) = parent {
                    parent = p.borrow().parent.as_ref().and_then(Weak::upgrade);
                    if p.borrow().name == "/" {
                        return Ok(());
                    }
                }
                Err(err!("Was not anchored to root"))
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
                for child in &from.clone().borrow().children {
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
        self.parent.as_ref()?.upgrade()
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

fn parse_input_arena(input: &str) -> Result<Arena> {
    let mut arena = Arena::new();
    let mut cwd: Option<ArenaIndex> = None;
    for line in input.lines() {
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["$", "cd", "/"] => {
                cwd = Some(arena.add_item(ArenaDir::new("/"), None));
            }
            ["$", "cd", dir] => {
                let Some(ref mut cwd) = cwd else {
                    return Err(err!("Attempt to cd while uninitialized"));
                };

                let cwd_dir = arena.dir_at(*cwd)?;

                let target = if *dir == ".." {
                    cwd_dir.parent.as_ref()
                } else {
                    cwd_dir
                        .children
                        .iter()
                        .find(|idx| arena.at(**idx).name() == *dir)
                }
                .ok_or_else(|| err!("unable to find child for cd to {dir}"))?;

                *cwd = *target;
            }
            ["$", "ls"] => (),
            ["dir", name] => {
                let Some(cwd) = cwd else {
                    return Err(err!("Attempt to add a dir with no cwd"));
                };
                let dir = ArenaDir::new(name);
                arena.add_item(dir, Some(cwd));
            }
            [num, name] if num.parse::<u32>().is_ok() => {
                let Some(cwd) = cwd else {
                    return Err(err!("Attempt to add a file with no cwd"));
                };
                let num = num.parse::<u32>().unwrap();
                let file = ArenaFile::new(name, num);
                arena.add_item(file, Some(cwd));
            }
            _ => return Err(err!("Unrecognized input: {}", line)),
        }
    }
    Ok(arena)
}

fn parse_input(input: &str) -> Result<Rc<RefCell<Dir>>> {
    let mut root = None;
    let mut cwd: Option<Rc<RefCell<Dir>>> = None;
    for line in input.lines() {
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

    fn try_for_each<T, F>(&self, predicate: &mut F) -> ControlFlow<T>
    where
        F: FnMut(&Self) -> ControlFlow<T>,
    {
        match self {
            f @ Item::File(_) => predicate(f)?,
            d @ Item::Dir(dir) => {
                predicate(d)?;
                for child in &dir.borrow().children {
                    child.borrow().try_for_each(predicate)?;
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn find_map<T, F>(&self, predicate: &mut F) -> Option<T>
    where
        F: FnMut(&Self) -> Option<T>,
    {
        let value = self.try_for_each(&mut |item| {
            if let Some(value) = predicate(item) {
                ControlFlow::Break(value)
            } else {
                ControlFlow::Continue(())
            }
        });
        match value {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(value) => Some(value),
        }
    }

    fn collect(&self) -> Vec<Self> {
        let mut v = Vec::new();
        self.find_map(&mut |node| {
            v.push(node.clone());
            None::<()>
        });
        v
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

fn part1_arena(root: &Arena, size_limit: u32) -> u32 {
    root.iter_indices()
        .filter_map(|idx| {
            let ArenaItem::Dir(_) = root.at(idx) else { return None };
            let size = root.size(idx);
            if size <= size_limit {
                Some(size)
            } else {
                None
            }
        })
        .sum()
}

fn part1_iter(root: &Item, size_limit: u32) -> u32 {
    root.iter()
        .filter_map(|i| match &*i.borrow() {
            i @ Item::Dir(_) if i.size() < size_limit => Some(i.size()),
            _ => None,
        })
        .sum::<u32>()
}

fn part1_internal_iter(root: &Item, size_limit: u32) -> u32 {
    let mut sum = 0;
    root.find_map(&mut |item| {
        let Item::Dir(_) = item else { return None::<()> };
        let size = item.size();
        if size <= size_limit {
            sum += size;
        };
        None::<()>
    });
    sum
}

fn part2_iter(root: &Item) -> Result<u32> {
    let (fs_size, free_min) = (70_000_000, 30_000_000);
    let used_space = root.size();

    let currently_free = fs_size - used_space;
    let needed = free_min - currently_free;

    root.iter()
        .filter_map(|item| match &*item.borrow() {
            Item::Dir(d) if d.borrow().size() < needed => None,
            Item::Dir(d) => Some(d.borrow().size()),
            Item::File(_) => None,
        })
        .min()
        .ok_or_else(|| err!("No sufficiently large directory found"))
}

fn part2_internal_iter(root: &Item) -> Result<u32> {
    let (fs_size, free_min) = (70_000_000, 30_000_000);
    let used_space = root.size();

    let currently_free = fs_size - used_space;
    let needed = free_min - currently_free;

    let mut result: Option<u32> = None;
    root.find_map(&mut |item| {
        match item {
            Item::Dir(d) if d.borrow().size() < needed => (),
            Item::Dir(d) => {
                let size = d.borrow().size();
                let Some(smallest) = result else {
                    result = Some(size);
                    return None::<()>;
                };
                if size < smallest {
                    result = Some(size);
                };
            }
            Item::File(_) => (),
        };
        None
    });
    result.ok_or_else(|| err!("No sufficiently large directory found"))
}

fn part2_arena(arena: &Arena) -> Result<u32> {
    let (fs_size, free_min) = (70_000_000, 30_000_000);
    let used_space = arena.size(0);

    let currently_free = fs_size - used_space;
    let needed = free_min - currently_free;

    arena
        .iter_indices()
        .filter_map(|idx| match arena.at(idx) {
            ArenaItem::Dir(_) if arena.size(idx) < needed => None,
            ArenaItem::Dir(_) => Some(arena.size(idx)),
            ArenaItem::File(_) => None,
        })
        .min()
        .ok_or_else(|| err!("No sufficiently large directory found"))
}

fn main() -> Result<()> {
    let parsed = Item::Dir(parse_input(INPUT)?);

    println!("day 07 part 1: {}", part1_iter(&parsed, 100_000));
    println!("day 07 part 2: {}", part2_iter(&parsed)?);

    // println!("day 07 part 1: {}", part1_internal_iter(&parsed, 100_000));
    // println!("day 07 part 2: {}", part2_internal_iter(&parsed)?);

    // let arena = parse_input_arena(include_str!("../input.txt"))?;
    // println!("day 07 part 1: {}", part1_arena(&arena, 100_000));
    // println!("day 07 part 2: {}", part2_arena(&arena)?);
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
        let root = Dir::new("/", Option::default());
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
        let root = Dir::new("/", Option::default());
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
        let root = Dir::new("/", Option::default());
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

        // Clone to prevent dropping the root
        let cwd = &mut root.clone();
        // root, a, d, e
        assert_eq!(cwd.borrow().size(), 48_381_165);
        Dir::cd(cwd, "d").unwrap();
        assert_eq!(cwd.borrow().size(), 24_933_642);
        Dir::cd(cwd, "..").unwrap();
        Dir::cd(cwd, "a").unwrap();
        assert_eq!(cwd.borrow().size(), 94853);
        Dir::cd(cwd, "e").unwrap();
        assert_eq!(cwd.borrow().size(), 584);
    }

    const ITER_RESULT: [&str; 14] = [
        "/", "a", "e", "i", "f", "g", "h.lst", "b.txt", "c.dat", "d", "j", "d.log", "d.ext", "k",
    ];

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
        let parsed = parse_input(EXAMPLE_INPUT).unwrap();
        assert_eq!(parsed.borrow().name(), "/");
        assert_eq!(parsed.borrow().children[0].borrow().name(), "a");
        assert_eq!(
            parsed.borrow().children.last().unwrap().borrow().name(),
            "d"
        );
    }

    #[test]
    fn test_add_child() {
        let root = Dir::new("/", Option::default());
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
        let root = Dir::new("/", Option::default());
        let child_dir = Dir::new("bar", None);
        let child_file = Rc::new(RefCell::new(Item::File(File::new("foo", 7))));
        let subdir_file = Rc::new(RefCell::new(Item::File(File::new("baz", 14))));

        Dir::add_child(&root, &child_file);
        Dir::add_child(&child_dir, &subdir_file);

        let child_dir = Rc::new(RefCell::new(Item::Dir(child_dir)));
        Dir::add_child(&root, &child_dir);
        assert!(child_dir.borrow().parent().is_some());

        // Clone to prevent dropping the root
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
    fn test_visit_with_iter() {
        let parsed = parse_input(EXAMPLE_INPUT).unwrap();

        let mut items = <Vec<String>>::new();
        Item::Dir(parsed).find_map(&mut |i| {
            items.push(i.name());
            None::<()>
        });

        assert_eq!(items, ITER_RESULT);
    }

    #[test]
    fn test_part1_iter() {
        let root = Item::Dir(parse_input(EXAMPLE_INPUT).unwrap());
        assert_eq!(part1_iter(&root, 100_000), 95437);
    }

    #[test]
    fn test_part1_internal_iter() {
        let root = Item::Dir(parse_input(EXAMPLE_INPUT).unwrap());
        assert_eq!(part1_internal_iter(&root, 100_000), 95437);
    }

    #[test]
    fn test_parse_arena() {
        let parsed = parse_input_arena(EXAMPLE_INPUT).unwrap();
        assert_eq!(parsed.at(0).name(), "/");
        assert_eq!(parsed.at(parsed.dir_at(0).unwrap().children[0]).name(), "a");
        assert_eq!(
            parsed
                .at(*parsed.dir_at(0).unwrap().children.last().unwrap())
                .name(),
            "d"
        );
    }

    #[test]
    fn test_arena_iter() {
        let parsed = parse_input_arena(EXAMPLE_INPUT).unwrap();
        let result: Vec<_> = parsed.iter().map(ArenaItem::name).collect();
        assert_eq!(result, ITER_RESULT);
    }

    #[test]
    fn test_part1_arena() {
        let parsed = parse_input_arena(EXAMPLE_INPUT).unwrap();
        assert_eq!(part1_arena(&parsed, 100_000), 95437);
    }

    #[test]
    fn test_part1_all_implementations() {
        let my_part1_solution = 1_517_599;
        let parsed = Item::Dir(parse_input(INPUT).unwrap());
        assert_eq!(part1_iter(&parsed, 100_000), my_part1_solution);
        assert_eq!(part1_internal_iter(&parsed, 100_000), my_part1_solution);
        assert_eq!(
            part1_arena(&parse_input_arena(INPUT).unwrap(), 100_000),
            my_part1_solution
        );
    }

    #[test]
    fn test_part2_iter() {
        let solution = 24_933_642;
        let parsed = Item::Dir(parse_input(EXAMPLE_INPUT).unwrap());
        assert_eq!(part2_iter(&parsed).unwrap(), solution);
    }

    #[test]
    fn test_part2_internal_iter() {
        let solution = 24_933_642;
        let parsed = Item::Dir(parse_input(EXAMPLE_INPUT).unwrap());
        assert_eq!(part2_internal_iter(&parsed).unwrap(), solution);
    }

    #[test]
    fn test_part2_arena() {
        let solution = 24_933_642;
        let arena = parse_input_arena(EXAMPLE_INPUT).unwrap();
        assert_eq!(part2_arena(&arena).unwrap(), solution);
    }
}

#[cfg(feature = "bench")]
mod benches {
    extern crate test;
    use super::*;
    use test::Bencher;

    const PART1_SOLUTION: u32 = 1_517_599;
    const PART2_SOLUTION: u32 = 2_481_982;

    #[bench]
    fn bench_parse(b: &mut Bencher) {
        b.iter(|| {
            let _parsed = Item::Dir(parse_input(INPUT).unwrap());
        })
    }

    #[bench]
    fn bench_part1_iter(b: &mut Bencher) {
        let parsed = Item::Dir(parse_input(INPUT).unwrap());
        b.iter(|| {
            assert_eq!(part1_iter(&parsed, 100_000), PART1_SOLUTION);
        })
    }

    #[bench]
    fn bench_part1_internal_iter(b: &mut Bencher) {
        let parsed = Item::Dir(parse_input(INPUT).unwrap());
        b.iter(|| {
            assert_eq!(part1_internal_iter(&parsed, 100_000), PART1_SOLUTION);
        })
    }

    #[bench]
    fn bench_parse_arena(b: &mut Bencher) {
        b.iter(|| {
            let _arena = parse_input_arena(INPUT).unwrap();
        })
    }

    #[bench]
    fn bench_part1_arena(b: &mut Bencher) {
        let arena = parse_input_arena(INPUT).unwrap();
        b.iter(|| {
            assert_eq!(part1_arena(&arena, 100_000), PART1_SOLUTION);
        })
    }

    #[bench]
    fn bench_part2_iter(b: &mut Bencher) {
        let parsed = Item::Dir(parse_input(INPUT).unwrap());
        b.iter(|| {
            assert_eq!(part2_iter(&parsed).unwrap(), PART2_SOLUTION);
        })
    }

    #[bench]
    fn bench_part2_internal_iter(b: &mut Bencher) {
        let parsed = Item::Dir(parse_input(INPUT).unwrap());
        b.iter(|| {
            assert_eq!(part2_internal_iter(&parsed).unwrap(), PART2_SOLUTION);
        })
    }

    #[bench]
    fn bench_part2_arena(b: &mut Bencher) {
        let arena = parse_input_arena(INPUT).unwrap();
        b.iter(|| {
            assert_eq!(part2_arena(&arena).unwrap(), PART2_SOLUTION);
        })
    }
}
