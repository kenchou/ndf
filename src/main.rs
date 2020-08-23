#![allow(non_snake_case)]

// use colored::*;
use sysinfo::{DiskExt, SystemExt};
use std::ffi::{OsString};

fn get_frac(avail: &u64, total: &u64) -> f64 {
    if *total == 0 {
        return 0 as f64;
    }

    let x = (avail/total) as f64;
    return x.ceil();
}

struct NDFDisk {
    name: String,
    space_asfrac: f64
}

impl NDFDisk {
    fn create_NDFDisk(disk: &sysinfo::Disk) -> NDFDisk {
        let frac = get_frac(&disk.get_available_space(), &disk.get_total_space());

        match OsString::from(disk.get_name()).into_string() {
            Ok(s) => {
                NDFDisk {
                    name: s,
                    space_asfrac: frac
                }
            },
            Err(_) => panic!("No name for disk.")
        }

    }
}

fn main() {
    let sys = sysinfo::System::new();
    let mut disks: Vec<NDFDisk> = Vec::new();
    for disk in sys.get_disks() {
        disks.push(NDFDisk::create_NDFDisk(disk));
    };

    for disk in disks.into_iter() {
        println!("{}, {:?}", disk.name, disk.space_asfrac);
    }
}