use chrono::prelude::*;
//use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use yaml_rust::Yaml;

const DEFAULT_MAX_AGE_HOURS: i64 = 30;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    pub running: bool,
    pub snapshot_statuses: HashMap<String, SnapshotStatus>,
}

impl SystemStatus {
    pub fn new(config: Arc<Option<Yaml>>) -> SystemStatus {
        SystemStatus {
            running: is_restic_running(),
            snapshot_statuses: get_new_status_map(config),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SnapshotStatus {
    last_check: i64,
    last_snapshot: i64,
    max_age_hours: i64,
}

impl SnapshotStatus {
    pub fn new(max_age_hours: i64) -> SnapshotStatus {
        SnapshotStatus {
            last_check: Local.ymd(2000, 1, 1).and_hms(12, 0, 0).timestamp(),
            last_snapshot: Local.ymd(2000, 1, 1).and_hms(12, 0, 0).timestamp(),
            max_age_hours,
        }
    }
    pub fn snapshot_current(&self) -> bool {
        (Local::now().timestamp() - self.last_snapshot) / 60 / 60 < self.max_age_hours
    }
    pub fn status_current(&self) -> bool {
        (Local::now().timestamp() - self.last_check) / 60 < 60
    }
}

pub fn cmd(backup_tag: &str, restic_args: Vec<&str>) {
    let backup = backup_from_tag(&backup_tag);

    if let Some(yml) = backup {
        run_set_env(&yml["env"]);

        let mut restic_cmd = Command::new(restic_binary());

        for arg in restic_args.iter() {
            restic_cmd.arg(arg);
        }

        match restic_cmd.output() {
            Ok(output) => {
                println!("{}", String::from_utf8(output.stdout).unwrap());
            }
            Err(e) => {
                eprintln!("Restic command failed: {:?}", e);
            }
        }

        run_clear_env(&yml["env"]);
    }
}

pub fn is_restic_running() -> bool {
    let no_procs = Vec::new();
    let process_rslt = &psutil::process::processes();

    let processes = match process_rslt {
        Ok(p) => p,
        Err(_e) => &no_procs,
    };

    for pr in processes {
        if let Ok(p) = pr {
            let cmdline = match p.cmdline() {
                Ok(c) => c,
                Err(_e) => Some(String::from("")),
            };
    
            if cmdline
                .unwrap_or_else(|| format!("[{}]", p.name().unwrap()))
                .to_lowercase()
                .contains("restic ")
            {
                return true;
            }
        }
    }
    false
}

pub fn restic_snapshot_status(backup_name: &str) -> SnapshotStatus {
    let backup = backup_from_name(backup_name);
    let mut snapshots = Vec::new();
    let mut max_age_hours: i64 = DEFAULT_MAX_AGE_HOURS;

    if let Some(yml) = backup {
        run_set_env(&yml["env"]);

        let mut restic_cmd = Command::new(restic_binary());

        restic_cmd.arg("snapshots");
        restic_cmd.arg("--json");
        restic_cmd.arg(format!("--tag=bungee-{}", &yml["tag"].as_str().unwrap()));

        match restic_cmd.output() {
            Ok(output) => {
                snapshots = output.stdout;
            }
            Err(e) => {
                eprintln!("Restic command failed: {:?}", e);
            }
        }

        run_clear_env(&yml["env"]);
        max_age_hours = yml["max_age_hours"].as_i64().unwrap();
    }

    let snapshots = str::from_utf8(&snapshots).unwrap();
    let snapshots: Vec<Value> = match serde_json::from_str(snapshots) {
        Ok(v) => v,
        Err(_e) => Vec::new(),
    };

    let mut status = SnapshotStatus::new(max_age_hours);

    for snapshot in snapshots {
        let ds = match &snapshot["time"] {
            Value::String(s) => s,
            _ => "",
        };

        match ds.parse::<DateTime<Local>>() {
            Ok(d) => {
                status.last_check = Local::now().timestamp();
                if d.timestamp() > status.last_snapshot {
                    status.last_snapshot = d.timestamp();
                }
            }
            Err(_e) => {}
        }
    }
    status
}

fn restic_binary() -> String {
    let bin = Path::new("/usr/bin/restic");
    if !bin.exists() {
        return String::from("/bin/restic")
    }
    String::from("/usr/bin/restic")
}

pub fn refresh_is_running(system_status: Arc<Mutex<SystemStatus>>) {
    let mut system_status = system_status.lock().unwrap();
    system_status.running = is_restic_running();
}

pub fn refresh_status(system_status: Arc<Mutex<SystemStatus>>, config: Arc<Option<Yaml>>) {
    let mut new_status = HashMap::new();
    for backup_name in get_backup_names(config.clone()) {
        new_status.insert(backup_name.clone(), restic_snapshot_status(&backup_name));
    }

    let mut system_status = system_status.lock().unwrap();
    for backup_name in get_backup_names(config.clone()) {
        let snapshot_entry = system_status
            .snapshot_statuses
            .entry(backup_name.clone())
            .or_insert(SnapshotStatus::new(DEFAULT_MAX_AGE_HOURS));
        if let Some(status) = new_status.get(&backup_name[..]) {
            snapshot_entry.last_check = status.last_check;
            snapshot_entry.last_snapshot = status.last_snapshot;
        }
    }
}

pub fn refresh_expired_backups(system_status: Arc<Mutex<SystemStatus>>, config: Arc<Option<Yaml>>) {
    let mut backup_queue = Vec::new();
    {
        let system_status_temp = system_status.lock().unwrap();
        for (backup_name, snapshot_status) in system_status_temp.snapshot_statuses.iter() {
            if !snapshot_status.snapshot_current() {
                backup_queue.push(backup_name.clone());
            }
        }
    }

    for backup_name in backup_queue.iter() {
        if !is_restic_running() {
            run(backup_name.clone());
            refresh_status(system_status.clone(), config.clone());
        }
    }
}

pub fn is_status_current(
    system_status: Arc<Mutex<SystemStatus>>,
    config: Arc<Option<Yaml>>,
) -> bool {
    let system_status = system_status.lock().unwrap();
    for backup_name in get_backup_names(config) {
        if let Some(status) = system_status.snapshot_statuses.get(&backup_name[..]) {
            if !status.status_current() {
                return false;
            }
        }
    }
    true
}

pub fn get_backup_names(config: Arc<Option<Yaml>>) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    if let Some(ref yml) = *config {
        for backup in yml.clone().into_iter() {
            list.push(String::from(backup["name"].as_str().unwrap()));
        }
    }
    list
}

pub fn get_new_status_map(config: Arc<Option<Yaml>>) -> HashMap<String, SnapshotStatus> {
    let mut backup_map = HashMap::new();
    for backup_name in get_backup_names(config) {
        let mut max_age_hours = DEFAULT_MAX_AGE_HOURS;
        if let Some(yml) = backup_from_name(&backup_name) {
            max_age_hours = yml["max_age_hours"].as_i64().unwrap();
        }
        backup_map.insert(backup_name, SnapshotStatus::new(max_age_hours));
    }
    backup_map
}

pub fn backup_from_name(backup_name: &str) -> Option<Yaml> {
    let all_backups = crate::get_config(crate::get_config_yaml());

    if let Some(yml) = all_backups {
        for backup in yml.into_iter() {
            if backup["name"].as_str().unwrap() == backup_name {
                return Some(backup);
            }
        }
    }
    None
}

pub fn backup_from_tag(backup_tag: &str) -> Option<Yaml> {
    let all_backups = crate::get_config(crate::get_config_yaml());

    if let Some(yml) = all_backups {
        for backup in yml.into_iter() {
            if backup["tag"].as_str().unwrap() == backup_tag {
                return Some(backup);
            }
        }
    }
    None
}

pub fn init_repo() {
    if let Err(_err) = Command::new(restic_binary()).arg("init").output() {
        println!("Restic repo init failed.");
    }
}

pub fn run(backup_name: String) {
    let config = backup_from_name(&backup_name);

    if let Some(yml) = config {
        run_set_env(&yml["env"]);

        init_repo();

        let mut restic_cmd = Command::new(restic_binary());
        restic_cmd.arg("backup");
        restic_cmd.arg(format!("--tag=bungee-{}", &yml["tag"].as_str().unwrap()));

        run_apply_excludes(&mut restic_cmd, &yml["exclude"]);
        run_apply_includes(&mut restic_cmd, &yml["include"]);

        if let Err(e) = restic_cmd.output() {
            eprintln!("Restic command failed: {:?}", e);
        }

        let mut restic_cmd = Command::new(restic_binary());
        restic_cmd.arg("forget");
        restic_cmd.arg("--keep-hourly 48");
        restic_cmd.arg("--keep-daily 60");
        restic_cmd.arg("--keep-monthly 48");
        restic_cmd.arg("--keep-yearly 99");
        restic_cmd.arg(format!("--tag=bungee-{}", &yml["tag"].as_str().unwrap()));

        if let Err(e) = restic_cmd.output() {
            eprintln!("Restic command failed: {:?}", e);
        }

        run_clear_env(&yml["env"]);
    }
}

fn run_apply_excludes(cmd: &mut Command, e: &Yaml) {
    if let Yaml::Array(a) = e {
        for i in a.iter() {
            if let Yaml::String(s) = i {
                cmd.arg(format!("--exclude={}", s));
            }
        }
    }
}

fn run_apply_includes(cmd: &mut Command, e: &Yaml) {
    if let Yaml::Array(a) = e {
        for i in a.iter() {
            if let Yaml::String(s) = i {
                cmd.arg(s);
            }
        }
    }
}

fn run_set_env(e: &Yaml) {
    if let Yaml::Hash(ref h) = e {
        for (k, v) in h {
            let mut var_name = &String::from("");
            let mut val = &String::from("");
            if let Yaml::String(vn) = k {
                var_name = vn
            };
            if let Yaml::String(va) = v {
                val = va
            };
            env::set_var(var_name, val);
        }
    }
}

fn run_clear_env(e: &Yaml) {
    if let Yaml::Hash(ref h) = e {
        for (k, _v) in h {
            let var_name = match k {
                Yaml::String(ref vn) => vn,
                _ => "",
            };
            env::set_var(var_name, "");
        }
    }
}
