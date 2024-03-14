// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut input: [u8; 1024] = [0; 1024];
                let read = stream.read(&mut input);
                match read {
                    Ok(n) => {
                        println!("read {} bytes", n);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
                let written = stream.write(b"HTTP/1.1 200 OK\r\n\r\n".as_ref());
                match written {
                    Ok(n) => {
                        println!("wrote {} bytes", n);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
