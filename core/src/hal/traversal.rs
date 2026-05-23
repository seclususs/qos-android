//! Author: [Seclususs](https://github.com/seclususs)

use std::{fs, os, path};

pub enum TraversalAction {
    Keep,
    DeleteFile,
    Stop,
}

fn open_dir_secure(path: &path::Path) -> Result<std::os::fd::OwnedFd, std::io::Error> {
    rustix::fs::openat(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))
}

#[inline]
fn get_safe_fd_path(fd_raw: os::fd::RawFd, path_buf: &mut [u8; 32]) -> &str {
    path_buf[0..14].copy_from_slice(b"/proc/self/fd/");

    let mut itoa_buf = itoa::Buffer::new();
    let fd_bytes = itoa_buf.format(fd_raw).as_bytes();
    let end = 14 + fd_bytes.len();

    path_buf[14..end].copy_from_slice(fd_bytes);

    unsafe { std::str::from_utf8_unchecked(&path_buf[..end]) }
}

pub fn get_tree_size_capped(dir: &path::Path, limit: u64, depth: usize) -> u64 {
    if depth > 20 {
        return 0;
    }

    let Ok(fd) = open_dir_secure(dir) else {
        return 0;
    };

    let mut path_buf = [0u8; 32];
    let safe_path = get_safe_fd_path(os::fd::AsRawFd::as_raw_fd(&fd), &mut path_buf);

    let Ok(entries) = fs::read_dir(safe_path) else {
        return 0;
    };

    let mut size = 0;

    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };

        if ft.is_symlink() {
            continue;
        }

        if ft.is_dir() {
            if size < limit {
                size += get_tree_size_capped(&entry.path(), limit - size, depth + 1);
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

    let Ok(fd) = open_dir_secure(dir) else {
        return 0;
    };

    let mut path_buf = [0u8; 32];
    let safe_path = get_safe_fd_path(os::fd::AsRawFd::as_raw_fd(&fd), &mut path_buf);

    let Ok(entries) = fs::read_dir(safe_path) else {
        return 0;
    };

    let mut count = 0;

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
