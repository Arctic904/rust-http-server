use std::{
    io::{BufRead, BufReader, Error, Read, Write},
    net::TcpListener,
    vec,
};

use itertools::Itertools;
use nom::InputIter;

/// Represents an HTTP request.
struct Request {
    method: String,
    path: String,
    version: String,
    headers: Headers,
    body: Option<String>,
}

/// Represents the headers of an HTTP request.
struct Headers {
    host: String,
    agent: String,
    others: Option<String>,
}

/// List of acceptable paths for the HTTP server.
const ACCEPTABLE_PATHS: [&str; 2] = ["/", "/echo"];

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let reader = BufReader::new(&mut stream);
                let line_reader = reader.lines();
                let mut input = String::new();
                for line in line_reader {
                    input = input.to_owned() + &line.as_ref().unwrap() + "\r\n";
                    if line.unwrap().as_bytes().is_empty() {
                        break;
                    }
                }
                println!("{}", input);
                let request = parse_request(input);
                let path = &request.path.as_str();

                println!("{}", path);

                let written = if path.starts_with("/echo/") {
                    let content = path.split_at("/echo/".len()).1;
                    stream.write(gen_response(200, "OK", Some((content, "text/plain"))).as_bytes())
                } else if path.starts_with("/user-agent") {
                    stream.write(
                        gen_response(200, "OK", Some((&request.headers.agent, "text/plain")))
                            .as_bytes(),
                    )
                } else if path == &"/" {
                    stream.write(gen_response(200, "OK", None).as_bytes())
                } else {
                    stream.write(gen_response(404, "Not Found", None).as_bytes())
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

/// Parses an HTTP request string and returns a `Request` struct.
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

/// Generates an HTTP response.
///
/// * `status` - HTTP Status Code
/// * `status_msg` - HTTP Message
/// * `content` - Optional (content, content_type)
///
fn gen_response(status: i16, status_msg: &str, content: Option<(&str, &str)>) -> String {
    let mut response = format!("HTTP/1.1 {} {}", status, status_msg);
    if let Some(content) = content {
        response += format!(
            "\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content.1,
            content.0.len(),
            content.0
        )
        .as_str();
    } else {
        response += "\r\n\r\n"
    }
    response
}
