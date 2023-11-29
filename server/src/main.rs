use std::io::prelude::*;
use std::os::fd::AsRawFd;
use std::{
    io::BufReader,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
};

use common::ReadPoller;

fn create_tcp_server(port: u16) -> TcpListener {
    let sa = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(sa).unwrap()
}

struct Client {
    stream: TcpStream,
    nickname: String,
}

struct ChatState {
    listener: TcpListener,
    clients: Vec<Client>,
}

fn main() {
    let mut chat_state = ChatState {
        listener: create_tcp_server(7701),
        clients: Vec::new(),
    };

    let mut poller = ReadPoller::new();

    loop {
        poller.add_read(&chat_state.listener);
        for client in &chat_state.clients {
            poller.add_read(&client.stream);
        }
        poller.poll().unwrap();

        if poller.is_readable(&chat_state.listener) {
            let (stream, _) = chat_state.listener.accept().unwrap();
            let mut reader = BufReader::new(&stream);

            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            let mut nickname = line.trim();
            if !nickname.starts_with("user:") {
                panic!("Invalid nickname: {}", nickname);
            }
            nickname = nickname.trim_start_matches("user:");

            let mut client = Client {
                stream,
                nickname: nickname.to_string(),
            };

            let welcome_msg = "Welcome to Simple Chat! Use /nick <nick> to set your nick.\n";
            client.stream.write_all(welcome_msg.as_bytes()).unwrap();

            println!("Connected client {}", client.nickname);
            chat_state.clients.push(client);
        }

        let mut clients_to_remove = Vec::new();
        for i in 0..chat_state.clients.len() {
            let (before, mid_after) = chat_state.clients.split_at_mut(i);
            let (client, after) = mid_after.split_first_mut().unwrap();

            if !poller.is_readable(&client.stream) {
                continue;
            }

            let mut reader = BufReader::new(&client.stream);
            let mut line = String::new();
            let nread = reader.read_line(&mut line).unwrap();
            if nread <= 0 {
                // disconnect
                println!(
                    "Disconnected client fd={}, nick={}",
                    client.stream.as_raw_fd(),
                    client.nickname
                );
                clients_to_remove.push(i);
                continue;
            }

            let line = line.trim();

            if line.starts_with("/") {
                let cmd = line.trim_start_matches("/");
                if cmd.starts_with("nick") {
                    let nickname = cmd.trim_start_matches("nick ").trim();
                    client.nickname = nickname.to_string();
                } else {
                    let msg = format!("Unknown command: {}\n", cmd.trim());
                    client.stream.write_all(msg.as_bytes()).unwrap();
                }
            } else {
                let msg = format!("{}> {}", client.nickname, line.trim());
                println!("{}", msg);
                for other_client in before.iter_mut().chain(after.iter_mut()) {
                    let msg = format!("{}\n", msg);
                    other_client.stream.write_all(msg.as_bytes()).unwrap();
                }
            }
        }

        for (i, client) in clients_to_remove.iter().enumerate() {
            chat_state.clients.remove(client - i);
        }

        poller.clear();
    }
}
