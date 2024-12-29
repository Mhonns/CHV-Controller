use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("154.215.14.240:2546").unwrap();
    // To add maximum number of thread and close up properly follow
    // https://doc.rust-lang.org/book/ch20-01-single-threaded.html

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            handle_connection(stream);
        });
    }   
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    // request_line = "GET /vm/create HTTP/1.1"

    let (status_line) = match &request_line[..] {
        "GET /vm/create HTTP/1.1" => ("HTTP/1.1 200 OK"),
        _ => ("HTTP/1.1 404 NOT FOUND"),
    };
    println!("{}", status_line);
}