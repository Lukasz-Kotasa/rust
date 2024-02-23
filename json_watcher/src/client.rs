use std::io::Read;

//from lib.rs
use json_watcher::PIPE_PATH;
//use json_watcher::{State, Field, Message};

extern crate unix_named_pipe;

fn main() {
    let mut pipe = unix_named_pipe::open_read(PIPE_PATH).expect("could not open pipe for reading");
    loop {
        let mut payload: [u8; 128] = [0; 128];
        let res = pipe.read(&mut payload);
        if let Ok(count) = res {
            if count > 0 {
                println!("got data ({}) from server:{}", count, std::str::from_utf8(&payload).unwrap());
            }
        }
    }
}