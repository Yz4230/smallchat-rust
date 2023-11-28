use std::{
    io::{prelude::*, stdin, BufReader, BufWriter},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
};

use common::ReadPoller;
use rand::distributions::{Alphanumeric, DistString};

fn connect_to_server(port: u16) -> TcpStream {
    let sa = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpStream::connect(sa).unwrap()
}

fn get_random_nickname() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
}

fn main() {
    let stream = connect_to_server(7701);
    let mut writer = BufWriter::new(&stream);

    let nickname = get_random_nickname();
    writer
        .write_all(format!("user:{}\n", nickname).as_bytes())
        .unwrap();
    writer.flush().unwrap();

    println!("Connected to server as {}", nickname);

    let mut poller = ReadPoller::new();
    let stdin = stdin();

    loop {
        poller.add_read(&stream);
        poller.add_read(&stdin);
        poller.poll().unwrap();

        if poller.is_readable(&stream) {
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            print!("{}", line);
        }

        if poller.is_readable(&stdin) {
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            writer.write_all(line.as_bytes()).unwrap();
            writer.flush().unwrap();
        }

        poller.reset();
    }
}
