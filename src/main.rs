#![allow(unused_imports)]
use std::{io::{Read, Write, Result}, net::TcpListener};







fn handle_client(mut stream:std::net::TcpStream){ 
        loop{
                let mut buf = [0u8; 512];
                 match stream.read(&mut buf){
                    Ok(n) => {
                        if n == 0 {
                            break;
                            }

                        let pong = String::from("+PONG\r\n");

                        stream.write_all(pong.as_bytes()).unwrap();
                    }
                    Err(error) => {
                        eprintln!("{:#?}", error);
                     }
                }
            }
}








fn main() -> Result<()> {

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let stream = stream?; 
        std::thread::spawn(move || handle_client(stream));
    }
    
    Ok(())
}
