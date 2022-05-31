use crate::ipc::{self, RequestType, ResponseType};
use crate::restic::SystemStatus;
use crate::server;
use bincode::deserialize;
use std::io;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub fn start(skip_daemon: bool) {
    let (tx_req, rx_res) = mpsc::channel();

    let systray_h = client_thread(tx_req);
    let _status_h = status_thread(rx_res, skip_daemon);

    systray_h.join().unwrap();
}

fn status_thread(rx_res: Receiver<RequestType>, skip_daemon: bool) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match UnixStream::connect("/tmp/bungee-backup.sock") {
            Ok(mut stream) => {
                handle_stream(&mut stream, &rx_res);
            }
            Err(e) => {
                eprintln!("Client - Failed to connect: {}", e);
                if !skip_daemon {
                    thread::spawn(server::start);
                }
            }
        }
        thread::sleep(Duration::from_secs(10));
    })
}

fn handle_response(response: &[u8]) {
    let response: ResponseType = deserialize(&response[..]).unwrap();
    match response {
        ResponseType::Status(system_status) => {
            process_status_response(system_status);
        }
        ResponseType::List(_list) => {}
    }
}

fn process_status_response(system_status: SystemStatus) {
    if system_status.running {
        println!("Staus: Syncing")
    } else {
        let mut warn = false;
        for (_, snapshot_status) in system_status.snapshot_statuses {
            if !snapshot_status.snapshot_current() {
                warn = true;
            }
        }
        if warn {
            println!("Status: Warning")
        } else {
            println!("Status: Okay")
        }
    }
}

fn handle_stream(stream: &mut UnixStream, rx_res: &Receiver<RequestType>) {
    let req_type = match rx_res.try_recv() {
        Ok(m) => match m {
            RequestType::Run(c) => RequestType::Run(c),
            RequestType::Check => RequestType::Check,
            RequestType::List => RequestType::List,
        },
        Err(_e) => RequestType::Check,
    };
    ipc::send_request(stream, req_type);
    handle_response(&ipc::receive_response(stream));
}

fn client_thread(tx_req: Sender<RequestType>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // get user input then send request to server
            if false {
                // run a backup
                tx_req
                    .send(RequestType::Run(String::from("Backup Name")))
                    .unwrap();

                // request list of backups
                tx_req.send(RequestType::List).unwrap();
            }

            thread::sleep(Duration::from_secs(10));
        }
    })
}

fn _get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_input) => {}
        Err(_err) => {}
    }
    input.trim().to_string()
}
