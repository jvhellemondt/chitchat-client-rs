use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::{io, thread};
use std::time::Duration;

const LOCAL_IP: &str = "127.0.0.1";
const PORT: &str = "6000";
const MSG_SIZE: usize = 32;

fn main() {
    let address = format!("{}:{}", LOCAL_IP.to_owned(), PORT.to_owned());
    let mut client: TcpStream = match TcpStream::connect(address.as_str()) {
        Ok(stream) => {
            println!("Client listening to {}", address);
            stream
        },
        Err(err) => panic!("Client failed to start, {}", err),
    };

    match client.set_nonblocking(true) {
        Ok(_) => println!("Client set to non-blocking"),
        Err(err) => panic!("Failed to set client to non-blocking, {}", err),
    }

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0;MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message received");
                println!("Channel: {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection to server was shut down");
                break;
            }
        }
        
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                println!("Message sent, {:?}", msg)
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message: ");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed!");
        let msg = buff.trim().to_string();
        if msg == ":q" || tx.send(msg).is_err() { break }
    }

    println!("Client shut down. Bye!")
}
