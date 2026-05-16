//! Author: [Seclususs](https://github.com/seclususs)

use crate::tweaks::system::FileTweak;

use std::{fs, path};

const NVME_PRIORITY: &[&str] = &["kyber", "mq-deadline", "none"];
const UFS_SSD_PRIORITY: &[&str] = &["mq-deadline", "kyber", "deadline", "none"];
const EMMC_PRIORITY: &[&str] = &["mq-deadline", "deadline", "noop", "none"];
const ROTATIONAL_PRIORITY: &[&str] = &["bfq", "mq-deadline", "deadline"];
const IGNORED_DEVICES: &[&str] = &["loop", "ram", "zram", "dm-", "md"];

fn is_device_rotational(dev_name: &str) -> bool {
    let path = format!("/sys/block/{dev_name}/queue/rotational");
    fs::read_to_string(path).is_ok_and(|s| s.trim() == "1")
}

fn select_scheduler_from_str(content: &str, priorities: &[&'static str]) -> Option<&'static str> {
    priorities
        .iter()
        .find(|&&candidate| content.contains(candidate))
        .copied()
}

fn get_device_priorities(name: &str) -> &'static [&'static str] {
    if is_device_rotational(name) {
        ROTATIONAL_PRIORITY
    } else if name.starts_with("nvme") {
        NVME_PRIORITY
    } else if name.starts_with("mmcblk") {
        EMMC_PRIORITY
    } else {
        UFS_SSD_PRIORITY
    }
}

pub fn generate_scheduler_tweaks() -> Vec<FileTweak> {
    let mut tweaks = Vec::new();
    let block_dir = path::Path::new("/sys/block");
    let Ok(entries) = fs::read_dir(block_dir) else {
        return tweaks;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if IGNORED_DEVICES
            .iter()
            .any(|&prefix| name.starts_with(prefix))
        {
            continue;
        }

        tweaks.push(FileTweak::new_dynamic(
            format!("/sys/block/{name}/queue/add_random"),
            "0",
        ));
        tweaks.push(FileTweak::new_dynamic(
            format!("/sys/block/{name}/queue/iostats"),
            "1",
        ));
        tweaks.push(FileTweak::new_dynamic(
            format!("/sys/block/{name}/queue/rq_affinity"),
            "1",
        ));

        let sched_path = format!("/sys/block/{name}/queue/scheduler");
        let Ok(content) = fs::read_to_string(&sched_path) else {
            continue;
        };

        let priorities = get_device_priorities(&name);
        let Some(selected) = select_scheduler_from_str(&content, priorities) else {
            continue;
        };

        tweaks.push(FileTweak::new_dynamic(sched_path, selected));

        match selected {
            "mq-deadline" | "deadline" => {
                tweaks.push(FileTweak::new_dynamic(
                    format!("/sys/block/{name}/queue/iosched/fifo_batch"),
                    "16",
                ));
                tweaks.push(FileTweak::new_dynamic(
                    format!("/sys/block/{name}/queue/iosched/writes_starved"),
                    "2",
                ));
                tweaks.push(FileTweak::new_dynamic(
                    format!("/sys/block/{name}/queue/iosched/front_merges"),
                    "1",
                ));
            }
            "bfq" => {
                tweaks.push(FileTweak::new_dynamic(
                    format!("/sys/block/{name}/queue/iosched/slice_idle"),
                    "0",
                ));
            }
            _ => {}
        }
    }
    tweaks
}
