#![allow(unused_imports)]
use bytes::{Buf, BytesMut};
use std::collections::HashMap;
use std::{
    io::{Read, Result, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const DELINEATOR: &[u8] = b"\r\n";
const COMMAND_FRONT: &[u8] = b"*";
const ARG_FRONT: &[u8] = b"$";

#[derive(Clone, Debug)]
struct Entry {
    value: Vec<u8>,
    expires: Option<Instant>,
}

type Table = Arc<Mutex<HashMap<Vec<u8>, Entry>>>;

fn parse_number<'a>(
    buffer: &'a [u8],
    front_byte: &[u8],
    delineator: &[u8],
) -> Option<(usize, &'a [u8])> {
    let n = buffer
        .windows(delineator.len())
        .position(|w| w == delineator)?;
    let (length_buf, remainder) = buffer.split_at(n);

    if !length_buf.starts_with(front_byte) {
        return None;
    }

    let length = std::str::from_utf8(&length_buf[1..])
        .ok()?
        .parse::<usize>()
        .ok()?;
    Some((length, &remainder[2..]))
}

fn arg_to_bulk_string<W: Write>(stream: &mut W, buffer: &[u8]) -> Result<()> {
    stream.write_all(ARG_FRONT)?;
    stream.write_all(format!("{}", buffer.len()).as_bytes())?;
    stream.write_all(DELINEATOR)?;
    stream.write_all(buffer)?;
    stream.write_all(DELINEATOR)?;

    Ok(())
}

fn parse(buffer: &[u8]) -> Option<(Vec<Vec<u8>>, usize)> {
    let parse_command_number = |buffer| parse_number(buffer, COMMAND_FRONT, b"\r\n");
    let parse_arg_number = |buffer| parse_number(buffer, ARG_FRONT, b"\r\n");

    let (num_args, mut args) = parse_command_number(buffer)?;
    let mut parsed_args: Vec<Vec<u8>> = vec![];
    let mut total_length = COMMAND_FRONT.len()
        + DELINEATOR.len()
        + (if let Some(num) = num_args.checked_ilog10() {
            num + 1
        } else {
            1
        }) as usize;

    for _ in 0..num_args {
        let (length, args_temp) = parse_arg_number(args)?;
        let arg = args_temp.get(0..length)?;
        parsed_args.push(Vec::from(arg));

        total_length += ARG_FRONT.len()
            + 2 * DELINEATOR.len()
            + (if let Some(num) = length.checked_ilog10() {
                num + 1
            } else {
                1
            }) as usize
            + length;
        args = &args_temp[(length + 2)..];
    }
    Some((parsed_args, total_length))
}

fn handle_client(mut stream: std::net::TcpStream, instance: Table) -> Result<()> {
    let mut commands: BytesMut = BytesMut::new();
    loop {
        let mut buf = [0u8; 512];
        let n = stream.read(&mut buf)?;

        if n == 0 {
            return Ok(());
        }

        commands.extend(&buf[..n]);
        if let Some((parsed, length)) = parse(&commands)
            && let Some((command, args)) = parsed.split_first()
        {
            match command.to_ascii_uppercase().as_slice() {
                b"ECHO" => {
                    if let Some(arg) = args.first() {
                        arg_to_bulk_string(&mut stream, arg)?;
                    }
                }
                b"PING" => {
                    stream.write_all(b"+PONG\r\n")?;
                }
                b"GET" => {
                    if let Some(arg) = args.first() {
                        {
                            let mut hash_map = instance.lock().unwrap();
                            match hash_map.get(arg).cloned() {
                                Some(entry) => match entry.expires {
                                    Some(num) if (Instant::now() >= num) => {
                                        hash_map.remove(arg);
                                        stream.write_all(b"$-1\r\n")?;
                                    }
                                    _ => {
                                        arg_to_bulk_string(&mut stream, &entry.value)?;
                                    }
                                },
                                None => {
                                    stream.write_all(b"$-1\r\n")?;
                                }
                            }
                        }
                    }
                }
                b"SET" => {
                    if let (Some(key), Some(value)) = (args.first(), args.get(1)) {
                        {
                            let mut hash_map = instance.lock().unwrap();

                            let expires_at: Option<Instant> = match (args.get(2), args.get(3)) {
                                (Some(measure), Some(t)) => {
                                    if let Some(num) =
                                        std::str::from_utf8(t).unwrap_or("").parse::<u64>().ok()
                                    {
                                        match measure.to_ascii_uppercase().as_slice() {
                                            b"PX" => {
                                                Some(Instant::now() + Duration::from_millis(num))
                                            }
                                            b"EX" => {
                                                Some(Instant::now() + Duration::from_secs(num))
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            let entry = Entry {
                                value: value.clone(),
                                expires: expires_at,
                            };

                            hash_map.insert(key.clone(), entry);

                            stream.write_all(b"+OK")?;
                            stream.write_all(DELINEATOR)?;
                        }
                    }
                }
                _ => {}
            }
            commands.advance(length);
        }
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let data_table: Table = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let instance = Arc::clone(&data_table);
        std::thread::spawn(move || handle_client(stream, instance));
    }

    Ok(())
}
