use colored::*;
use std::env;
use sysinfo::Disks;

const MAX_CHARS: usize = 50;

fn get_frac(avail: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    1.0 - (avail as f64 / total as f64)
}

struct NDFDisk {
    name: String,
    space_asfrac: f64,
    mnt: String,
}

impl NDFDisk {
    fn create_ndf_disk(disk: &sysinfo::Disk) -> NDFDisk {
        let frac = get_frac(disk.available_space(), disk.total_space());
        NDFDisk {
            name: disk.name().to_string_lossy().to_string(),
            space_asfrac: frac,
            mnt: disk.mount_point().to_string_lossy().to_string(),
        }
    }
    fn create_bar(&self) -> ColoredString {
        let chars_num = (MAX_CHARS as f64 * self.space_asfrac).ceil() as usize;
        let chars = "▓".repeat(chars_num);
        let rem_num = MAX_CHARS - chars_num;
        let rem = "░".repeat(rem_num);

        if rem_num < (MAX_CHARS as f64 * 0.2) as usize {
            format!("{}{}", chars, rem).red()
        } else {
            format!("{}{}", chars, rem).green()
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut compact_mode = false;

    if let Some(val) = args.get(1) {
        if val == "compact" {
            compact_mode = true;
        }
    }

    let mut disks: Vec<NDFDisk> = Vec::new();
    for disk in Disks::new_with_refreshed_list().list() {
        // ignore overlay and snap mounts
        if disk.file_system() == "overlay" || disk.mount_point().starts_with("/var/snap/") {
            continue;
        }
        disks.push(NDFDisk::create_ndf_disk(disk));
    }

    println!("{}", "\nndf - nice disk free".bold());

    if compact_mode {
        for disk in disks.into_iter() {
            println!(
                "{}: {} {:.0}%",
                disk.name,
                disk.create_bar(),
                disk.space_asfrac * 100.0
            );
        }
    } else {
        for disk in disks.into_iter() {
            println!(
                "{} @ {}\n{} {:.0}%\n",
                disk.name,
                disk.mnt,
                disk.create_bar(),
                disk.space_asfrac * 100.0
            );
        }
    }
}
