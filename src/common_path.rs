#[cfg(test)]
extern crate rand;

#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

pub fn common_path_all(paths: Vec<PathBuf>) -> Option<PathBuf> {
    let mut path_iter = paths.into_iter();
    let mut result = path_iter.next()?.to_path_buf();
    for path in path_iter {
        if let Some(r) = common_path(result, path) {
            result = r;
        } else {
            return None;
        }
    }
    Some(result.to_path_buf())
}

pub fn common_path(one: PathBuf, two: PathBuf) -> Option<PathBuf> {
    let one = one;
    let two = two;
    let one = one.components();
    let two = two.components();
    let mut final_path = PathBuf::new();
    let mut found = false;
    let paths = one.zip(two);
    for (l, r) in paths {
        if l == r {
            final_path.push(l.as_os_str());
            found = true;
        } else {
            break;
        }
    }
    if found {
        Some(final_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{seq::SliceRandom, thread_rng};

    #[test]
    fn compare_all_paths() {
        let mut rng = thread_rng();
        for _ in 0..6 {
            let one = Path::new("/foo/bar/baz/one.txt").to_path_buf();
            let two = Path::new("/foo/bar/quux/quuux/two.txt").to_path_buf();
            let three = Path::new("/foo/bar/baz/foo/bar").to_path_buf();
            let result = Path::new("/foo/bar");
            let mut all = vec![one, two, three];
            all.shuffle(&mut rng);
            assert_eq!(common_path_all(all).unwrap(), result.to_path_buf())
        }
    }

    #[test]
    fn compare_paths() {
        let one = Path::new("/foo/bar/baz/one.txt").to_path_buf();
        let two = Path::new("/foo/bar/quux/quuux/two.txt").to_path_buf();
        let result = Path::new("/foo/bar").to_path_buf();
        assert_eq!(common_path(one, two).unwrap(), result)
    }

    #[test]
    fn no_common_path() {
        let one = Path::new("/foo/bar").to_path_buf();
        let two = Path::new("./baz/quux").to_path_buf();
        assert!(common_path(one, two).is_none());
    }
}
