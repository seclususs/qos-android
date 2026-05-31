//! Author: [Seclususs](https://github.com/seclususs)

use std::{ffi, os, path};

pub enum TraversalAction {
    Keep,
    DeleteFile,
    Stop,
}

pub fn get_tree_size_capped(dir: &path::Path, limit: u64, depth: usize) -> u64 {
    if depth > 20 {
        return 0;
    }

    let Ok(fd) = rustix::fs::openat(
        rustix::fs::CWD,
        dir,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    ) else {
        return 0;
    };

    get_tree_size_capped_raw(os::fd::AsFd::as_fd(&fd), limit, depth)
}

fn get_tree_size_capped_raw(dir_fd: os::fd::BorrowedFd<'_>, limit: u64, depth: usize) -> u64 {
    if depth > 20 {
        return 0;
    }

    let Ok(dir_clone) = os::fd::BorrowedFd::try_clone_to_owned(&dir_fd) else {
        return 0;
    };

    let Ok(mut dir_iter) = rustix::fs::Dir::read_from(dir_clone) else {
        return 0;
    };

    let mut size = 0;

    while let Some(Ok(entry)) = dir_iter.next() {
        let name_c = entry.file_name();

        let name_bytes = name_c.to_bytes();

        if name_bytes == b"." || name_bytes == b".." {
            continue;
        }

        let ft = entry.file_type();

        let mut is_dir = ft == rustix::fs::FileType::Directory;
        let mut is_symlink = ft == rustix::fs::FileType::Symlink;
        let mut known_size = None;

        if ft == rustix::fs::FileType::Unknown
            && let Ok(stat) = rustix::fs::statx(
                dir_fd,
                name_c,
                rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
                rustix::fs::StatxFlags::TYPE | rustix::fs::StatxFlags::SIZE,
            )
        {
            let mode = stat.stx_mode as u32 & 0o170_000;
            is_dir = mode == 0o040_000;
            is_symlink = mode == 0o120_000;

            if !is_dir && !is_symlink {
                known_size = Some(stat.stx_size);
            }
        }

        if is_symlink {
            continue;
        }

        if is_dir {
            if size < limit
                && let Ok(sub_fd) = rustix::fs::openat(
                    dir_fd,
                    name_c,
                    rustix::fs::OFlags::RDONLY
                        | rustix::fs::OFlags::DIRECTORY
                        | rustix::fs::OFlags::NOFOLLOW
                        | rustix::fs::OFlags::CLOEXEC,
                    rustix::fs::Mode::empty(),
                )
            {
                size +=
                    get_tree_size_capped_raw(os::fd::AsFd::as_fd(&sub_fd), limit - size, depth + 1);
            }
        } else if let Some(s) = known_size {
            size += s;
        } else if let Ok(stat) = rustix::fs::statx(
            dir_fd,
            name_c,
            rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
            rustix::fs::StatxFlags::SIZE,
        ) {
            size += stat.stx_size;
        }

        if size > limit {
            return size;
        }
    }

    size
}

pub fn walk_and_act<F>(dir: &path::Path, callback: &mut F, depth: usize) -> usize
where
    F: FnMut(os::fd::BorrowedFd<'_>, &ffi::CStr, rustix::fs::FileType, usize) -> TraversalAction,
{
    if depth > 20 {
        return 0;
    }

    let Ok(fd) = rustix::fs::openat(
        rustix::fs::CWD,
        dir,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    ) else {
        return 0;
    };

    walk_and_act_raw(os::fd::AsFd::as_fd(&fd), callback, depth)
}

fn walk_and_act_raw<F>(dir_fd: os::fd::BorrowedFd<'_>, callback: &mut F, depth: usize) -> usize
where
    F: FnMut(os::fd::BorrowedFd<'_>, &ffi::CStr, rustix::fs::FileType, usize) -> TraversalAction,
{
    if depth > 20 {
        return 0;
    }

    let Ok(dir_clone) = os::fd::BorrowedFd::try_clone_to_owned(&dir_fd) else {
        return 0;
    };

    let Ok(mut dir_iter) = rustix::fs::Dir::read_from(dir_clone) else {
        return 0;
    };

    let mut count = 0;

    while let Some(Ok(entry)) = dir_iter.next() {
        let name_c = entry.file_name();

        let name_bytes = name_c.to_bytes();

        if name_bytes == b"." || name_bytes == b".." {
            continue;
        }

        let ft = entry.file_type();

        let mut is_dir = ft == rustix::fs::FileType::Directory;
        let mut is_symlink = ft == rustix::fs::FileType::Symlink;

        if ft == rustix::fs::FileType::Unknown
            && let Ok(stat) = rustix::fs::statx(
                dir_fd,
                name_c,
                rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
                rustix::fs::StatxFlags::TYPE,
            )
        {
            let mode = stat.stx_mode as u32 & 0o170_000;
            is_dir = mode == 0o040_000;
            is_symlink = mode == 0o120_000;
        }

        if is_symlink {
            continue;
        }

        if is_dir {
            if let Ok(sub_fd) = rustix::fs::openat(
                dir_fd,
                name_c,
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            ) {
                count += walk_and_act_raw(os::fd::AsFd::as_fd(&sub_fd), callback, depth + 1);
            }
        } else {
            match callback(dir_fd, name_c, ft, depth) {
                TraversalAction::DeleteFile => {
                    if rustix::fs::unlinkat(dir_fd, name_c, rustix::fs::AtFlags::empty()).is_ok() {
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
