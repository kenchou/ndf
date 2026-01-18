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

// Define a struct to hold disk usage information
struct DiskUsage {
    total: u64,
    free: u64,
}

// Get disk usage for a given path using standard library
fn get_disk_usage_for_path(path: &str) -> std::io::Result<DiskUsage> {
    use std::mem;

    #[cfg(unix)]
    {
        use std::ffi::CString;

        let c_path = CString::new(path.as_bytes())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;

        let mut statvfs_buf: libc::statvfs = unsafe { mem::zeroed() };
        let result = unsafe {
            libc::statvfs(c_path.as_ptr(), &mut statvfs_buf)
        };

        if result == 0 {
            Ok(DiskUsage {
                total: (statvfs_buf.f_frsize as u64) * (statvfs_buf.f_blocks as u64),
                free: (statvfs_buf.f_frsize as u64) * (statvfs_buf.f_bavail as u64),
            })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    #[cfg(not(unix))]
    {
        // For non-Unix systems, we'll return an error for now
        // In a full implementation, we would use platform-specific APIs like GetDiskFreeSpaceEx on Windows
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Platform not supported"))
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
    let mut processed_mounts = std::collections::HashSet::new();

    // Process disks from sysinfo first
    for disk in Disks::new_with_refreshed_list().list() {
        let mnt = disk.mount_point().to_string_lossy();
        processed_mounts.insert(mnt.to_string());

        let fs_type = disk.file_system().to_string_lossy().to_string();

        // ignore overlay, devfs and snap mounts, but allow CIFS/SMB mounts
        if fs_type == "overlay" || fs_type == "devfs" || mnt.starts_with("/var/snap/") {
            continue;
        }

        // Skip system internal mount points that are not meaningful to users
        if mnt.starts_with("/private/") || mnt.starts_with("/sys/") ||
           mnt.starts_with("/proc/") || mnt.starts_with("/run/") ||
           mnt.starts_with("/boot/") || mnt.starts_with("/var/") && mnt != "/var" {
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

    // Now get all mount points using mountpoints crate to catch network mounts that sysinfo might miss
    if let Ok(mount_paths) = mountpoints::mountpaths() {
        for mount_path in mount_paths {
            let mount_str = mount_path.to_string_lossy().to_string();

            // Skip if already processed by sysinfo
            if processed_mounts.contains(&mount_str) {
                continue;
            }

            // Attempt to get disk usage for this mount point
            if let Ok(_metadata) = std::fs::metadata(&mount_path) {
                if let Ok(usage) = get_disk_usage_for_path(&mount_str) {

                    // Apply filters
                    if let Some(ref only) = only_mp {
                        if !only.contains(&mount_str) {
                            continue;
                        }
                    }
                    if let Some(ref exclude) = exclude_mp {
                        if exclude.contains(&mount_str) {
                            continue;
                        }
                    }

                    // Skip system internal mount points
                    if mount_str.starts_with("/System/Volumes/") && mount_str != "/System/Volumes/Data" ||
                       mount_str.starts_with("/dev/") || mount_str.starts_with("/private/") ||
                       mount_str.starts_with("/sys/") || mount_str.starts_with("/proc/") ||
                       mount_str.starts_with("/run/") || mount_str.starts_with("/boot/") ||
                       mount_str.starts_with("/var/snap/") || mount_str.starts_with("/var/") && mount_str != "/var" {
                        continue;
                    }

                    // Additionally, skip virtual filesystems that might be detected by mountpoints
                    // Check if the mount point is accessible and represents actual storage
                    if mount_str == "/dev" {
                        continue;  // Skip /dev which is a virtual filesystem
                    }

                    // Handle special cases for network mounts where usage stats might be unreliable
                    if usage.total == 0 && usage.free == 0 {
                        // Skip mount points that report zero bytes for both
                    } else if usage.total > 0 && usage.free <= usage.total {
                        // Regular case: total >= free
                        // Create an NDFDisk for this mount point
                        let frac = get_frac(usage.free, usage.total);
                        let mount_name = mount_str.split('/').last().unwrap_or("<unknown>").to_string();
                        disks.push(NDFDisk {
                            name: mount_name,
                            space_as_frac: frac,
                            mnt: mount_str.clone(),
                            size: usage.total,
                            free: usage.free,
                        });
                    } else if usage.total > 0 && usage.free > usage.total {
                        // Special case: free > total (common with network shares)
                        // We'll try to get more accurate info by checking if the path is accessible
                        if std::path::Path::new(&mount_str).exists() {
                            // For network mounts where free > total, we'll show the total as reported
                            // but calculate usage differently - maybe show as low usage
                            let frac = 0.1; // Assume low usage if free > total
                            let mount_name = mount_str.split('/').last().unwrap_or("<unknown>").to_string();
                            disks.push(NDFDisk {
                                name: mount_name,
                                space_as_frac: frac,
                                mnt: mount_str.clone(),
                                size: usage.total,
                                free: usage.free,
                            });
                        }
                    }
                }
            }
        }
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
            // 计算每列的最大宽度
            let mut max_mount_len = "Mount".len();
            let mut max_size_len = "Size".len();
            let mut max_free_len = "Free".len();
            let mut max_name_len = "Name".len();

            for disk in &disks {
                max_mount_len = max_mount_len.max(disk.mnt.len().min(20));
                max_size_len = max_size_len.max(format_size(disk.size).len());
                max_free_len = max_free_len.max(format_size(disk.free).len());
                max_name_len = max_name_len.max(disk.name.len().min(15));
            }

            // Usage列固定为进度条宽度 + 百分比
            let usage_len = MAX_CHARS + 4; // 50字符进度条 + 空格 + 3字符百分比

            // 手动创建表格
            println!(
                "┌{:─<width_mount$}┬{:─<width_size$}┬{:─<width_free$}┬{:─<width_usage$}┬{:─<width_name$}┐",
                "", "", "", "", "",
                width_mount = max_mount_len + 2,
                width_size = max_size_len + 2,
                width_free = max_free_len + 2,
                width_usage = usage_len + 2,
                width_name = max_name_len + 2
            );
            println!(
                "│ {:<width_mount$} │ {:>width_size$} │ {:>width_free$} │ {:^width_usage$} │ {:<width_name$} │",
                "Mount", "Size", "Free", "Usage", "Name",
                width_mount = max_mount_len,
                width_size = max_size_len,
                width_free = max_free_len,
                width_usage = usage_len,
                width_name = max_name_len
            );
            println!(
                "├{:─<width_mount$}┼{:─<width_size$}┼{:─<width_free$}┼{:─<width_usage$}┼{:─<width_name$}┤",
                "", "", "", "", "",
                width_mount = max_mount_len + 2,
                width_size = max_size_len + 2,
                width_free = max_free_len + 2,
                width_usage = usage_len + 2,
                width_name = max_name_len + 2
            );

            for disk in disks {
                let mount_text = if disk.mnt.len() > 20 {
                    disk.mnt[..17].to_string() + "..."
                } else {
                    disk.mnt.clone()
                };
                let size_text = format_size(disk.size);
                let free_text = format_size(disk.free);
                let name_text = if disk.name.len() > 15 {
                    disk.name[..12].to_string() + "..."
                } else {
                    disk.name.clone()
                };

                // 构建Usage列内容
                let plain_bar = disk.create_plain_bar();
                let percentage = format!("{:.0}%", disk.space_as_frac * 100.0);

                let colored_bar = if disk.is_high_usage() {
                    plain_bar.red()
                } else {
                    plain_bar.green()
                };

                println!(
                    "│ {:<width_mount$} │ {:>width_size$} │ {:>width_free$} │ {} {:>3} │ {:<width_name$} │",
                    mount_text, size_text, free_text, colored_bar, percentage, name_text,
                    width_mount = max_mount_len,
                    width_size = max_size_len,
                    width_free = max_free_len,
                    width_name = max_name_len
                );
            }

            println!(
                "└{:─<width_mount$}┴{:─<width_size$}┴{:─<width_free$}┴{:─<width_usage$}┴{:─<width_name$}┘",
                "", "", "", "", "",
                width_mount = max_mount_len + 2,
                width_size = max_size_len + 2,
                width_free = max_free_len + 2,
                width_usage = usage_len + 2,
                width_name = max_name_len + 2
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
