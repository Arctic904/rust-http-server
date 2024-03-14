use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Error, Read, Write},
    net::TcpListener,
    str::from_utf8,
    thread, vec,
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

    let dir: String = env::args()
        .skip_while(|arg| arg != "--directory")
        .next_tuple()
        .map_or(String::from("."), |(_, value)| value);

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        let dir_path = dir.clone();
        thread::spawn(move || match stream {
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
                    println!("{}", request.headers.agent);
                    stream.write(
                        gen_response(
                            200,
                            "OK",
                            Some((
                                &request
                                    .headers
                                    .agent
                                    .clone()
                                    .split_at(request.headers.agent.find(": ").unwrap() + 2)
                                    .1,
                                "text/plain",
                            )),
                        )
                        .as_bytes(),
                    )
                } else if path.starts_with("/files/") {
                    let path_to_file = path.split_at("/files/".len()).1;
                    let fp = dir_path + path_to_file;
                    let file: Result<File, Error> = File::open(fp);
                    let res;
                    match file {
                        Ok(mut file) => {
                            let mut buf: Vec<u8> = Vec::new();
                            file.read_to_end(&mut buf);
                            let content = from_utf8(&buf).unwrap();
                            res =
                                gen_response(200, "OK", Some((content, "application/octet-stream")))
                        }
                        Err(_) => res = gen_response(404, "Not Found", None),
                    }
                    stream.write(res.as_bytes())
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
        });
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
    println!("{}", response);
    response
}
