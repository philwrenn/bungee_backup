use crate::restic::SystemStatus;
use bincode::serialize;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestType {
    Check,
    Run(String),
    List,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseType {
    Status(SystemStatus),
    List(Vec<String>),
}

pub fn bytes_to_i64(b: [u8; 8]) -> i64 {
    (&b[..]).read_i64::<LittleEndian>().unwrap()
}

pub fn i64_to_bytes(i: i64) -> [u8; 8] {
    let mut bs = [0u8; 8];
    bs.as_mut()
        .write_i64::<LittleEndian>(i)
        .expect("Unable to write");
    bs
}

pub fn receive_request(stream: &mut UnixStream) -> Vec<u8> {
    let mut len = [0 as u8; 8];
    let mut received: i64 = 0;
    let mut buff = [0 as u8; 1024];
    let mut data = Cursor::new(Vec::new());

    let len = match stream.read_exact(&mut len) {
        Ok(_) => bytes_to_i64(len),
        Err(_) => {
            eprintln!("Server - An error occurred, terminating connection.");
            stream.shutdown(Shutdown::Both).unwrap();
            0
        }
    };

    while match stream.read(&mut buff) {
        Ok(size) => {
            data.write_all(&buff[0..size]).unwrap();
            received += size as i64;
            received < len
        }
        Err(_) => {
            eprintln!("Server - An error occurred, terminating connection.");
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}

    data.get_ref().to_owned()
}

pub fn receive_response(stream: &mut UnixStream) -> Vec<u8> {
    let mut len = [0 as u8; 8];
    let mut received: i64 = 0;
    let mut buff = [0 as u8; 1024];
    let mut data = Cursor::new(Vec::new());

    let len = match stream.read_exact(&mut len) {
        Ok(_) => bytes_to_i64(len),
        Err(e) => {
            eprintln!("Client - Failed to receive data: {}", e);
            0
        }
    };

    while match stream.read(&mut buff) {
        Ok(size) => {
            data.write_all(&buff[0..size]).unwrap();
            received += size as i64;
            received < len
        }
        Err(e) => {
            eprintln!("Client - Failed to receive data {}", e);
            false
        }
    } {}

    data.get_ref().to_owned()
}

pub fn send_request(stream: &mut UnixStream, req: RequestType) {
    let req = serialize(&req).unwrap();
    stream.write_all(&i64_to_bytes(req.len() as i64)).unwrap();
    stream.write_all(&req[..]).unwrap();
}

pub fn send_response(stream: &mut UnixStream, res: ResponseType) {
    let res = serialize(&res).unwrap();
    stream.write_all(&i64_to_bytes(res.len() as i64)).unwrap();
    stream.write_all(&res[..]).unwrap();
}
