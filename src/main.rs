use clap::{value_parser, Arg, Command, ValueEnum};
use colored::*;
use std::collections::HashSet;
use sysinfo::Disks;

const MAX_CHARS: usize = 50;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputMode {
    Normal,
    Compact,
    Table,
}

fn get_frac(avail: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    1.0 - (avail as f64 / total as f64)
}

struct NDFDisk {
    name: String,
    space_as_frac: f64,
    mnt: String,
    size: u64,
    free: u64,
}

impl NDFDisk {
    fn create_ndf_disk(disk: &sysinfo::Disk) -> NDFDisk {
        let frac = get_frac(disk.available_space(), disk.total_space());
        NDFDisk {
            name: disk.name().to_string_lossy().to_string(),
            space_as_frac: frac,
            mnt: disk.mount_point().to_string_lossy().to_string(),
            size: disk.total_space(),
            free: disk.available_space(),
        }
    }

    fn create_bar(&self) -> ColoredString {
        let chars_num = (MAX_CHARS as f64 * self.space_as_frac).ceil() as usize;
        let chars = "▓".repeat(chars_num);
        let rem_num = MAX_CHARS - chars_num;
        let rem = "░".repeat(rem_num);

        if rem_num < (MAX_CHARS as f64 * 0.2) as usize {
            format!("{}{}", chars, rem).red()
        } else {
            format!("{}{}", chars, rem).green()
        }
    }

    fn create_plain_bar(&self) -> String {
        let chars_num = (MAX_CHARS as f64 * self.space_as_frac).ceil() as usize;
        let chars = "▓".repeat(chars_num);
        let rem_num = MAX_CHARS - chars_num;
        let rem = "░".repeat(rem_num);
        format!("{}{}", chars, rem)
    }

    fn is_high_usage(&self) -> bool {
        let rem_num = MAX_CHARS - (MAX_CHARS as f64 * self.space_as_frac).ceil() as usize;
        rem_num < (MAX_CHARS as f64 * 0.2) as usize
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    match size {
        s if s >= TB => format!("{:.2}T", s as f64 / TB as f64),
        s if s >= GB => format!("{:.2}G", s as f64 / GB as f64),
        s if s >= MB => format!("{:.2}M", s as f64 / MB as f64),
        s if s >= KB => format!("{:.2}K", s as f64 / KB as f64),
        _ => format!("{}B", size),
    }
}

fn main() {
    let matches = Command::new("ndf")
        .about("Nice disk free.")
        .arg(
            Arg::new("mode")
                .value_parser(value_parser!(OutputMode))
                .default_value("table")
                .help("Display mode: normal | compact | table"),
        )
        .arg(
            Arg::new("only-mp")
                .long("only-mp")
                .value_name("MOUNTPOINTS")
                .help("Show only specified mount points, comma separated"),
        )
        .arg(
            Arg::new("exclude-mp")
                .long("exclude-mp")
                .value_name("MOUNTPOINTS")
                .help("Exclude specified mount points, comma separated"),
        )
        .get_matches();

    let output_mode = *matches.get_one::<OutputMode>("mode").unwrap();

    let only_mp: Option<HashSet<_>> = matches
        .get_one::<String>("only-mp")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    let exclude_mp: Option<HashSet<_>> = matches
        .get_one::<String>("exclude-mp")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    let mut disks: Vec<NDFDisk> = Vec::new();
    for disk in Disks::new_with_refreshed_list().list() {
        let mnt = disk.mount_point().to_string_lossy();
        // ignore overlay and snap mounts
        if disk.file_system() == "overlay" || mnt.starts_with("/var/snap/") {
            continue;
        }
        if let Some(ref only) = only_mp {
            if !only.contains(mnt.as_ref()) {
                continue;
            }
        }
        if let Some(ref exclude) = exclude_mp {
            if exclude.contains(mnt.as_ref()) {
                continue;
            }
        }
        disks.push(NDFDisk::create_ndf_disk(disk));
    }

    println!("{}", "ndf - nice disk free".bold());

    match output_mode {
        OutputMode::Compact => {
            for disk in disks {
                println!(
                    "{}: {} {:.0}%",
                    disk.name,
                    disk.create_bar(),
                    disk.space_as_frac * 100.0
                );
            }
        }
        OutputMode::Table => {
            // 手动创建表格以正确处理颜色
            println!(
                "┌{:─<22}┬{:─<10}┬{:─<10}┬{:─<56}┬{:─<14}┐",
                "", "", "", "", ""
            );
            println!(
                "│ {:<20} │ {:>8} │ {:>8} │ {:^54} │ {:<12} │",
                "Mount", "Size", "Free", "Usage", "Name"
            );
            println!(
                "├{:─<22}┼{:─<10}┼{:─<10}┼{:─<56}┼{:─<14}┤",
                "", "", "", "", ""
            );

            for disk in disks {
                let mount_col = format!(
                    "│ {:<20} │",
                    if disk.mnt.len() > 20 {
                        disk.mnt[..17].to_string() + "..."
                    } else {
                        disk.mnt.clone()
                    }
                );
                let size_col = format!(" {:>8} │", format_size(disk.size));
                let free_col = format!(" {:>8} │", format_size(disk.free));
                let name_col = format!(
                    " {:<12} │",
                    if disk.name.len() > 12 {
                        disk.name[..9].to_string() + "..."
                    } else {
                        disk.name.clone()
                    }
                );

                // 构建Usage列内容：1空格 + 50字符进度条 + 1空格 + 3字符百分比 + 1空格 = 56字符
                // 但我们用{:>3}格式化百分比，实际上是：1空格 + 50字符进度条 + 1空格 + 右对齐3字符百分比 + 1空格 = 55字符
                let plain_bar = disk.create_plain_bar();
                let percentage = format!("{:.0}%", disk.space_as_frac * 100.0);

                let colored_bar = if disk.is_high_usage() {
                    plain_bar.red()
                } else {
                    plain_bar.green()
                };

                let usage_final = format!(" {} {:>3} │", colored_bar, percentage);

                // 手动打印每行，不使用格式化来处理颜色部分
                print!("{}{}{}", mount_col, size_col, free_col);
                print!("{}", usage_final);
                println!("{}", name_col);
            }

            println!(
                "└{:─<22}┴{:─<10}┴{:─<10}┴{:─<56}┴{:─<14}┘",
                "", "", "", "", ""
            );
        }
        OutputMode::Normal => {
            for disk in disks {
                println!(
                    "{} @ {}\n{} {:.0}%\n",
                    disk.name,
                    disk.mnt,
                    disk.create_bar(),
                    disk.space_as_frac * 100.0
                );
            }
        }
    }
}
