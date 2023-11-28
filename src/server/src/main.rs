use std::io::prelude::*;
use std::{
    io::BufReader,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
};

fn create_tcp_server(port: u16) -> TcpListener {
    let sa = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(sa).unwrap()
}

fn accept_client(server_socket: &TcpListener) -> TcpStream {
    let (stream, _) = server_socket.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    stream.set_nodelay(true).unwrap();
    stream
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

    loop {
        let client_stream = accept_client(&chat_state.listener);
        let mut reader = BufReader::new(&client_stream);

        // first line is the nickname, format: user:<nickname>
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        let mut nickname = line.trim();
        if !nickname.starts_with("user:") {
            panic!("Invalid nickname: {}", nickname);
        }
        nickname = nickname.trim_start_matches("user:");

        let client = Client {
            stream: client_stream,
            nickname: nickname.to_string(),
        };

        let ip = client.stream.peer_addr().unwrap().ip().to_string();
        println!("New client: {} ({})", client.nickname, ip);
        chat_state.clients.push(client);
    }
}
