#![allow(unused_imports)]
use std::{io::{Read, Write}, net::TcpListener};

fn main() {
    // Anything you print shows up in Kiln's log as [your-program].

    //
     let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
     for stream in listener.incoming() {
         match stream {
             Ok(mut tcp_stream) => {
                let mut buf = [0u8; 512];
                 match tcp_stream.read(&mut buf){
                    Ok(n) => {
                        let message = &buf[..n];
                        let pong = String::from("+PONG\r\n");

                        tcp_stream.write_all(pong.as_bytes()).unwrap();



                    }
                    Err(error) => {}
                }

            },
            Err(e) => println!("error: {e}"),   }
     }
}
