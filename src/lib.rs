use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub mod client;
pub mod ipc;
pub mod restic;
pub mod server;

pub fn get_config_yaml() -> String {
    match fs::read_to_string("/etc/bungee-backup.yml") {
        Ok(s) => s,
        Err(_e) => String::from(""),
    }
}

pub fn get_config(yaml_string: String) -> Option<Yaml> {
    match YamlLoader::load_from_str(&yaml_string) {
        Ok(docs) => {
            if !docs.is_empty() {
                Some(docs[0].clone())
            } else {
                eprintln!("Yml config doc is empty.");
                None
            }
        }
        Err(_e) => {
            eprintln!("Error parsing Yml doc.");
            None
        }
    }
}
