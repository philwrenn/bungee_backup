use crate::ipc::{self, RequestType, ResponseType};
use crate::restic::{self, SystemStatus};
use bincode::deserialize;
use libc;
use std::ffi::CString;
use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use yaml_rust::Yaml;

pub fn start() {
    let config = Arc::new(crate::get_config(crate::get_config_yaml()));
    let system_status = Arc::new(Mutex::new(SystemStatus::new(config.clone())));

    {
        let thread_config = config.clone();
        let thread_system_status = system_status.clone();
        thread::spawn(|| status_refresh(thread_system_status, thread_config));
    }

    let listener = get_listener().unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let thread_config = config.clone();
                let thread_snapshot_status = system_status.clone();
                thread::spawn(move || handle_client(stream, thread_snapshot_status, thread_config));
            }
            Err(e) => {
                eprintln!("Server - Error: {}", e);
            }
        }
    }
    drop(listener);
}

fn get_listener() -> io::Result<UnixListener> {
    if let Err(_e) = fs::remove_file("/tmp/bungee-backup.sock") {};
    let listener = UnixListener::bind("/tmp/bungee-backup.sock")?;
    let sock_path = CString::new("/tmp/bungee-backup.sock").unwrap();

    unsafe {
        // todo: replace static group value
        libc::chown(sock_path.as_ptr(), 0, 1000);
        libc::chmod(sock_path.as_ptr(), 0o775);
    }
    Ok(listener)
}

fn handle_client(
    mut stream: UnixStream,
    system_status: Arc<Mutex<SystemStatus>>,
    config: Arc<Option<Yaml>>,
) {
    let req = ipc::receive_request(&mut stream);
    let req: RequestType = deserialize(&req[..]).unwrap();

    restic::refresh_is_running(system_status.clone());

    match req {
        RequestType::Check => {
            ipc::send_response(
                &mut stream,
                ResponseType::Status(system_status.lock().unwrap().clone()),
            );
        }
        RequestType::Run(c) => {
            let run_h = thread::spawn(move || {
                restic::run(c);
            });
            ipc::send_response(
                &mut stream,
                ResponseType::Status(system_status.lock().unwrap().clone()),
            );
            run_h.join().unwrap();
        }
        RequestType::List => {
            ipc::send_response(
                &mut stream,
                ResponseType::List(restic::get_backup_names(config.clone())),
            );
        }
    }
}

fn status_refresh(system_status: Arc<Mutex<SystemStatus>>, config: Arc<Option<Yaml>>) {
    loop {
        if !restic::is_status_current(system_status.clone(), config.clone()) {
            restic::refresh_status(system_status.clone(), config.clone());
        }

        restic::refresh_expired_backups(system_status.clone(), config.clone());
        thread::sleep(Duration::from_secs(60));
    }
}
