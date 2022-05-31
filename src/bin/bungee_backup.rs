use bungee_backup::client;
use bungee_backup::restic;
use bungee_backup::server;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut daemon = false;
    let mut skip_daemon = false;
    let mut backup = false;
    let mut backup_name = "";
    let mut restic_args = Vec::<&str>::new();

    for arg in args.iter() {
        match arg.to_lowercase().as_str() {
            "-d" | "--daemon-only" => daemon = true,
            "-s" | "--skip-daemon" => skip_daemon = true,
            "-b" | "--backup" => backup = true,
            _ => {
                if backup {
                    backup_name = arg;
                    backup = false;
                } else if backup_name != "" {
                    restic_args.push(arg);
                }
            }
        }
    }

    if backup_name != "" {
        restic::cmd(backup_name, restic_args);
    } else if daemon {
        server::start();
    } else {
        client::start(skip_daemon);
    }
}
