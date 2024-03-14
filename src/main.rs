// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

struct Request {
    method: String,
    path: String,
    version: String,
    headers: Headers,
    body: Option<String>,
}

struct Headers {
    host: String,
    agent: String,
    others: Option<String>,
}

const ACCEPTABLE_PATHS: [&str; 1] = ["/"];

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
                let input_utf8 = String::from_utf8(input.to_vec());
                let input_str = match input_utf8 {
                    Ok(s) => s,
                    Err(e) => {
                        println!("error: {}", e);
                        continue;
                    }
                };
                let request = parse_request(input_str);

                let written = if ACCEPTABLE_PATHS.contains(&request.path.as_str()) {
                    stream.write(b"HTTP/1.1 200 OK\r\n\r\n".as_ref())
                } else {
                    stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n".as_ref())
                };

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

fn parse_request(req: String) -> Request {
    let mut req = req.clone();
    let rest: String = req.split_off(req.find("\r\n").unwrap_or(req.len()));
    let inputs: Vec<&str> = req.split(' ').collect();
    let mut path: String = String::new();
    let mut version: String = String::new();
    let mut method: String = String::new();
    if inputs.len() == 3 {
        method = inputs[0].to_string();
        path = inputs[1].to_string();
        version = inputs[2].to_string();
    }
    let mut headers = rest.split("\r\n");
    // let headers: Vec<&str> = headers.collect();

    let headers = Headers {
        host: headers
            .find(|x| x.starts_with("Host:"))
            .unwrap_or("Unknown")
            .to_string(),
        agent: headers
            .find(|x| x.starts_with("User-Agent:"))
            .unwrap_or("Unknown")
            .to_string(),
        others: None,
    };
    Request {
        method,
        path,
        version,
        headers,
        body: None,
    }
}
