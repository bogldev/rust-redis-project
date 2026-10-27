#![allow(unused_imports)]
use std::{io::{Read, Write, Result}, net::TcpListener};
use bytes::{Buf,BytesMut};

const DELINEATOR:&[u8] = b"\r\n";
const COMMAND_FRONT: &[u8] = b"*";
const ARG_FRONT: &[u8] = b"$";

fn parse_number<'a>(buffer:&'a [u8], front_byte: &[u8], delineator: &[u8]) -> Option<(usize,&'a [u8])>{
    
    let n = buffer.windows(delineator.len()).position(|w| w == delineator)?;
    let (length_buf, remainder) = buffer.split_at(n);
    
    if !length_buf.starts_with(front_byte){
        return None;
    }


    let length = std::str::from_utf8(&length_buf[1..]).ok()?.parse::<usize>().ok()?;
//    let m = buffer.windows(delineator.len()).position(|w| w == delineator)?;
//    if m != 2 {
//        return None;
//    }

    Some((length, &remainder[2..]))
}


fn parse(buffer:&[u8]) -> Option<(Vec<Vec<u8>>, usize)>{
    
    let parse_command_number  = |buffer| parse_number(buffer, COMMAND_FRONT, b"\r\n"); 
    let parse_arg_number      = |buffer| parse_number(buffer, ARG_FRONT, b"\r\n");

    let (num_args, mut args) = parse_command_number(buffer)?;
    let mut parsed_args:Vec<Vec<u8>> = vec![]; 
    let mut total_length =  COMMAND_FRONT.len() + DELINEATOR.len() + (num_args.ilog10() + 1) as usize;
    
    for _ in 0..num_args {
      let (length,args_temp) = parse_arg_number(args)?;
      let arg = args_temp.get(0..length)?;
      parsed_args.push(Vec::from(arg));

    

      //let m = &args_temp[length..].windows(DELINEATOR.len()).position(|w| w == DELINEATOR)?;
    
      //if *m != 0 {
        //return None;
      //}

      total_length += ARG_FRONT.len() + 2*DELINEATOR.len() + (length.ilog10()  + 1) as usize + length;
      args = &args_temp[(length+2)..];

    }

    Some((parsed_args, total_length))

}


fn handle_client(mut stream:std::net::TcpStream) -> Result<()>{ 
        let mut commands:BytesMut = BytesMut::new();
        loop{
                let mut buf = [0u8; 512];
                let n = stream.read(&mut buf)?;

                if n == 0{
                    return Ok(());
                }

                commands.extend(&buf[..n]);
               if let Some((parsed, length)) = parse(&commands) && let Some ((command, args)) = parsed.split_first(){
                 match command.to_ascii_uppercase().as_slice()
                {
                 b"ECHO" => {
                    if let Some(arg) = args.first() {
                        stream.write_all(ARG_FRONT)?;
                        stream.write_all(format!("{}", arg.len()).as_bytes())?;
                        stream.write_all(DELINEATOR)?;
                        stream.write_all(arg)?;
                        stream.write_all(DELINEATOR)?;
                    }


                }
                 b"PING" => {

                    stream.write_all(b"+PONG\r\n")?;

                }
                _ => {}
                }
                commands.advance(length);
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
