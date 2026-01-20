//! Author: [Seclususs](https://github.com/seclususs)

use std::fs::{self, DirEntry};
use std::path::Path;

pub enum TraversalAction {
    Keep,
    DeleteFile,
    Stop,
}

pub fn get_tree_size_capped(path: &Path, limit: u64) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    if size < limit {
                        size += get_tree_size_capped(&entry.path(), limit - size);
                    }
                } else if ft.is_file()
                    && let Ok(meta) = entry.metadata()
                {
                    size += meta.len();
                }
            }
            if size > limit {
                return size;
            }
        }
    }
    size
}

pub fn walk_and_act<F>(dir: &Path, callback: &F, depth: usize) -> usize
where
    F: Fn(&DirEntry, usize) -> TraversalAction,
{
    if depth > 20 {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_symlink() {
                    continue;
                }
                if ft.is_dir() {
                    count += walk_and_act(&entry.path(), callback, depth + 1);
                    let _ = fs::remove_dir(entry.path());
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
        }
    }
    count
}