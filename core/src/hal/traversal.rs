//! Author: [Seclususs](https://github.com/seclususs)

use std::{fs, path};

pub enum TraversalAction {
    Keep,
    DeleteFile,
    Stop,
}

pub fn get_tree_size_capped(path: &path::Path, limit: u64) -> u64 {
    let mut size = 0;
    let Ok(entries) = fs::read_dir(path) else {
        return size;
    };

    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };

        if ft.is_dir() {
            if size < limit {
                size += get_tree_size_capped(&entry.path(), limit - size);
            }
        } else if ft.is_file() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            size += meta.len();
        }

        if size > limit {
            return size;
        }
    }
    size
}

pub fn walk_and_act<F>(dir: &path::Path, callback: &mut F, depth: usize) -> usize
where
    F: FnMut(&fs::DirEntry, usize) -> TraversalAction,
{
    if depth > 20 {
        return 0;
    }
    let mut count = 0;
    let Ok(entries) = fs::read_dir(dir) else {
        return count;
    };

    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };

        if ft.is_symlink() {
            continue;
        }

        if ft.is_dir() {
            count += walk_and_act(&entry.path(), callback, depth + 1);
        } else {
            match callback(&entry, depth) {
                TraversalAction::DeleteFile => {
                    if fs::remove_file(entry.path()).is_ok() {
                        count += 1;
                    }
                }
                TraversalAction::Stop => return count,
                TraversalAction::Keep => {}
            }
        }
    }
    count
}
