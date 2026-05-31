//! Author: [Seclususs](https://github.com/seclususs)

use crate::bindings::sys;
use crate::controllers::cleaner;
use crate::hal::traversal;

use std::{ffi, fs, os, path, sync, time};

const MALLOPT_TRIM: i32 = -101;
const SYSTEM_DUMP_PATHS: &[&str] = &["/data/anr", "/data/tombstones"];
const APP_DATA_PATHS: &[&str] = &["/data/data", "/sdcard/Android/data"];

pub struct CleanerWorker {
    pub config: cleaner::CleanerConfig,
    pub sweep_rx: sync::mpsc::Receiver<bool>,
}

impl CleanerWorker {
    pub fn new(config: cleaner::CleanerConfig, sweep_rx: sync::mpsc::Receiver<bool>) -> Self {
        Self { config, sweep_rx }
    }

    pub fn run(&mut self) {
        while let Ok(is_emergency) = self.sweep_rx.recv() {
            let items = self.perform_cycle(is_emergency);
            if items > 0 {
                log::debug!("Cycle complete. Removed {items} items.");
            }
            unsafe {
                sys::mallopt(MALLOPT_TRIM, 0);
            }
        }
    }

    #[inline]
    fn is_safe_name(name_bytes: &[u8]) -> bool {
        const SAFE_EXTS: &[&[u8]] = &[
            b".db",
            b".xml",
            b".obb",
            b".pak",
            b".dat",
            b".json",
            b".lock",
            b".pref",
            b".conf",
            b"-journal",
            b"-wal",
            b"-shm",
        ];
        SAFE_EXTS.iter().any(|ext| name_bytes.ends_with(ext))
    }

    #[inline]
    fn is_trash_extension(name_bytes: &[u8]) -> bool {
        const TRASH_EXTS: &[&[u8]] = &[
            b".tmp", b".temp", b".log", b".bak", b".old", b".thumb", b".exo",
        ];
        TRASH_EXTS.iter().any(|ext| name_bytes.ends_with(ext))
    }

    fn perform_cycle(&mut self, is_emergency: bool) -> usize {
        let now = time::SystemTime::now();

        let current_sec = now
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .cast_signed();

        let mut total_cleaned = 0;

        total_cleaned += self.clean_system_paths(current_sec);
        total_cleaned += self.clean_app_caches(is_emergency, current_sec);

        total_cleaned
    }

    fn clean_system_paths(&self, current_sec: i64) -> usize {
        let mut cleaned = 0;

        let trash_age_sec = self.config.age_trash.as_secs().cast_signed();
        let stale_media_sec = self.config.age_stale_media.as_secs().cast_signed();

        for system_path in SYSTEM_DUMP_PATHS {
            let dir_path = path::Path::new(system_path);

            let mut policy = |dir_fd: os::fd::BorrowedFd<'_>,
                              name_c: &ffi::CStr,
                              _ftype: rustix::fs::FileType,
                              _depth: usize|
             -> traversal::TraversalAction {
                let name_bytes = name_c.to_bytes();

                if Self::is_safe_name(name_bytes) {
                    return traversal::TraversalAction::Keep;
                }

                let threshold_sec = if Self::is_trash_extension(name_bytes) {
                    trash_age_sec
                } else {
                    stale_media_sec
                };

                if let Ok(stat) = rustix::fs::statx(
                    dir_fd,
                    name_c,
                    rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
                    rustix::fs::StatxFlags::MTIME,
                ) {
                    let age_sec = current_sec.saturating_sub(stat.stx_mtime.tv_sec);

                    if age_sec > threshold_sec {
                        return traversal::TraversalAction::DeleteFile;
                    }
                }

                traversal::TraversalAction::Keep
            };

            cleaned += traversal::walk_and_act(dir_path, &mut policy, 0);
        }
        cleaned
    }

    fn clean_app_caches(&self, is_storage_critical: bool, current_sec: i64) -> usize {
        let mut cleaned = 0;

        for root in APP_DATA_PATHS {
            let root_path = path::Path::new(root);

            let Ok(entries) = fs::read_dir(root_path) else {
                continue;
            };

            let mut path_buffer = root_path.to_path_buf();

            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };

                if !file_type.is_dir() {
                    continue;
                }

                path_buffer.push(entry.file_name());
                cleaned +=
                    self.process_app_directory(&mut path_buffer, is_storage_critical, current_sec);
                path_buffer.pop();
            }
        }
        cleaned
    }

    fn process_app_directory(
        &self,
        path_buffer: &mut path::PathBuf,
        is_storage_critical: bool,
        current_sec: i64,
    ) -> usize {
        let mut cleaned = 0;

        path_buffer.push("cache");
        cleaned += self.process_cache_dir(path_buffer, is_storage_critical, current_sec);
        path_buffer.pop();

        path_buffer.push("code_cache");
        cleaned += self.process_code_cache_dir(path_buffer, is_storage_critical, current_sec);
        path_buffer.pop();

        cleaned
    }

    fn process_cache_dir(
        &self,
        cache_dir: &path::Path,
        is_storage_critical: bool,
        current_sec: i64,
    ) -> usize {
        let cache_size_bytes = if is_storage_critical {
            0
        } else {
            traversal::get_tree_size_capped(cache_dir, self.config.bloat_limit_bytes + 1024, 0)
        };

        let target_age = if is_storage_critical {
            self.config.age_emergency
        } else if cache_size_bytes > self.config.bloat_limit_bytes {
            self.config.age_bloat
        } else {
            self.config.age_stale_media
        };

        let target_age_sec = target_age.as_secs().cast_signed();
        let trash_age_sec = self.config.age_trash.as_secs().cast_signed();

        let mut policy = |dir_fd: os::fd::BorrowedFd<'_>,
                          name_c: &ffi::CStr,
                          _ftype: rustix::fs::FileType,
                          _depth: usize|
         -> traversal::TraversalAction {
            let name_bytes = name_c.to_bytes();

            if !is_storage_critical && Self::is_safe_name(name_bytes) {
                return traversal::TraversalAction::Keep;
            }

            let threshold_sec = if Self::is_trash_extension(name_bytes) {
                trash_age_sec
            } else {
                target_age_sec
            };

            if let Ok(stat) = rustix::fs::statx(
                dir_fd,
                name_c,
                rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
                rustix::fs::StatxFlags::MTIME,
            ) {
                let age_sec = current_sec.saturating_sub(stat.stx_mtime.tv_sec);

                if age_sec > threshold_sec {
                    return traversal::TraversalAction::DeleteFile;
                }
            }

            traversal::TraversalAction::Keep
        };

        traversal::walk_and_act(cache_dir, &mut policy, 0)
    }

    fn process_code_cache_dir(
        &self,
        code_dir: &path::Path,
        is_storage_critical: bool,
        current_sec: i64,
    ) -> usize {
        let target_age = if is_storage_critical {
            self.config.age_emergency
        } else {
            self.config.age_stale_code
        };

        let target_age_sec = target_age.as_secs().cast_signed();
        let trash_age_sec = self.config.age_trash.as_secs().cast_signed();

        let mut policy = |dir_fd: os::fd::BorrowedFd<'_>,
                          name_c: &ffi::CStr,
                          _ftype: rustix::fs::FileType,
                          _depth: usize|
         -> traversal::TraversalAction {
            let name_bytes = name_c.to_bytes();

            if !is_storage_critical && Self::is_safe_name(name_bytes) {
                return traversal::TraversalAction::Keep;
            }

            let threshold_sec = if Self::is_trash_extension(name_bytes) {
                trash_age_sec
            } else {
                target_age_sec
            };

            if let Ok(stat) = rustix::fs::statx(
                dir_fd,
                name_c,
                rustix::fs::AtFlags::SYMLINK_NOFOLLOW,
                rustix::fs::StatxFlags::MTIME,
            ) {
                let age_sec = current_sec.saturating_sub(stat.stx_mtime.tv_sec);

                if age_sec > threshold_sec {
                    return traversal::TraversalAction::DeleteFile;
                }
            }

            traversal::TraversalAction::Keep
        };

        traversal::walk_and_act(code_dir, &mut policy, 0)
    }
}
