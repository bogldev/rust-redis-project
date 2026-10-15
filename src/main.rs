#![allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // Anything you print shows up in Kiln's log as [your-program].
    println!("Logs from your program will appear here!");

    // Section 1 asks for a server on port 6379. One way to start:
    //
    // let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(_stream) => println!("accepted new connection"),
    //         Err(e) => println!("error: {e}"),
    //     }
    // }
}
