use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Error, Read, Write},
    net::TcpListener,
    str::from_utf8,
    thread,
};

use itertools::Itertools;

/// Represents an HTTP request.
#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    // version: String,
    headers: Headers,
    body: Option<String>,
}

/// Represents the headers of an HTTP request.
#[derive(Debug)]
struct Headers {
    host: String,
    agent: String,
    content_length: usize,
    // others: Option<String>,
}

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
                // let reader = BufReader::new(&mut stream);
                let mut buf_reader = BufReader::new(&mut stream);
                let input: String = buf_reader
                    .by_ref()
                    .lines()
                    .map(|res| res.unwrap())
                    .take_while(|line| !line.is_empty())
                    .join("\r\n");
                println!("{}", input);

                let request = parse_request(input);
                let path = &request.path.as_str();

                println!("{}", path);

                let written;
                if request.method == "GET" {
                    written = if path.starts_with("/echo/") {
                        let content = path.split_at("/echo/".len()).1;
                        stream.write(
                            gen_response(200, "OK", Some((content, "text/plain"))).as_bytes(),
                        )
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
                        let res = match file {
                            Ok(mut file) => {
                                let mut buf: Vec<u8> = Vec::new();
                                let _ = file.read_to_end(&mut buf);
                                let content = from_utf8(&buf).unwrap();
                                gen_response(200, "OK", Some((content, "application/octet-stream")))
                            }
                            Err(_) => gen_response(404, "Not Found", None),
                        };
                        println!("{}", res);
                        stream.write(res.as_bytes())
                    } else if path == &"/" {
                        stream.write(gen_response(200, "OK", None).as_bytes())
                    } else {
                        stream.write(gen_response(404, "Not Found", None).as_bytes())
                    };
                } else if request.method == "POST" {
                    let mut buffer = vec![0; request.headers.content_length];
                    println!("{:?}", buffer);
                    let _ = buf_reader
                        .read_exact(&mut buffer)
                        .map_err(|err| println!("Error reading file: {:?}", err));
                    written = if request.path.starts_with("/files/") {
                        let mut fileparts: Vec<&str> = request.path.split('/').collect_vec();
                        let file_name = fileparts.pop().unwrap();
                        let no_files = fileparts.split_first().unwrap().1;
                        let temp_path = dir_path + "/" + &no_files.join("/");
                        let dir_made = fs::create_dir_all(&temp_path);
                        match dir_made {
                            Ok(_) => {
                                let fileres =
                                    fs::write(format!("{}/{}", temp_path, file_name), buffer);
                                match fileres {
                                    Ok(_) => {
                                        stream.write(gen_response(201, "CREATED", None).as_bytes())
                                    }
                                    Err(_) => stream
                                        .write(gen_response(404, "Not Found", None).as_bytes()),
                                }
                            }
                            Err(_) => stream.write(gen_response(404, "Not Found", None).as_bytes()),
                        }
                    } else {
                        stream.write(gen_response(404, "Not Found", None).as_bytes())
                    };
                } else {
                    written = stream.write(gen_response(404, "Not Found", None).as_bytes())
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
    println!("{}", req);
    let rest: String = req.split_off(req.find("\r\n").unwrap_or(req.len()));
    let inputs: Vec<&str> = req.split(' ').collect();
    let mut path: String = String::new();
    // let mut version: String = String::new();
    let mut method: String = String::new();
    if inputs.len() == 3 {
        method = inputs[0].to_string();
        path = inputs[1].to_string();
        // version = inputs[2].to_string();
    }
    let mut headers = rest.split("\r\n");

    let content = headers
        .clone()
        .find(|x| x.starts_with("Content-Length:"))
        .unwrap_or("");

    // println!(
    //     "\n\nREST:\n{}\n\nHEADERS:\n{:?}",
    //     rest,
    //     headers.clone().collect::<Vec<&str>>()
    // );

    let headers = Headers {
        host: headers
            .find(|x| x.starts_with("Host:"))
            .unwrap_or("Unknown")
            .to_string(),
        agent: headers
            .find(|x| x.starts_with("User-Agent:"))
            .unwrap_or("Unknown")
            .to_string(),
        content_length: content
            .split("Content-Length: ")
            .last()
            .unwrap_or("0")
            .parse::<usize>()
            .unwrap_or(0),
        // others: None,
    };

    let body = req.split_at(req.find("\r\n\r\n").unwrap_or_default());
    println!("body: {}", body.1);
    let body = if !body.1.is_empty() {
        Some(body.1.to_string())
    } else {
        None
    };

    println!("{}", headers.host);
    Request {
        method,
        path,
        // version,
        headers,
        body,
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
