#![allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    // Anything you print shows up in Kiln's log as [your-program].

    //
     let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
     for stream in listener.incoming() {
         match stream {
             Ok(_stream) => println!("accepted new connection"),
            Err(e) => println!("error: {e}"),   }
     }
}
