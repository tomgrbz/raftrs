const BROADCAST: &str = "FFFF";
use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let rep = Replica::new(cli.port, cli.id, cli.others);
    rep.run();
    
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    port: u32,
    id: String,
    others: Vec<String>,
}

struct Replica {
    port: u32,
    id: String,
    others: Vec<String>,
    socket: TcpListener,
}

impl Replica {
    fn new(port: u32, id: String, others: Vec<String>) -> Self {
        let socket = TcpListener::bind("localhost:0").unwrap();
        let others: Vec<String> = others.into_iter().map(|element| element.into()).collect();
        Replica {
            port,
            id: id.into(),
            others,
            socket,
        }
    }

    fn send(&mut self, message: &str) -> Result<(), String>{
        match self.socket.accept() {
            Ok((mut tcp, _sock)) => {
                tcp.write_all(message.as_bytes());
                Ok(())
            },
            Err(_) => {
                println!("Could not send message on socket.");
                Err("Could not send message.".into())
            }
        }
    }
    fn run(self) {
        let mut vec_buf: Vec<u8> = Vec::new();
        for stream in self.socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    match stream.read_to_end(&mut vec_buf) {
                        Ok(buf_size) => println!("Read {buf_size} amount of bytes"),
                        Err(e) => println!("Encountered error: {e}")
                    }
                },
                Err(e) => println!("Encountered error: {e}")
            };
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::Replica;


    #[test]
    fn test_new_replica() {
        Replica::new(9090, "id", vec!["1209", "1231"]);
    }
}