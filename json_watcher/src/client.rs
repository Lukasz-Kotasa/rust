use std::io::Read;
use serde_json::Value;

//from lib.rs
use json_watcher::PIPE_PATH;
use json_watcher::{State, Message};

extern crate unix_named_pipe;

fn parse_json_from_server(json_str: &str) {
    let json = serde_json::from_str::<Value>(json_str);

    if json.is_ok() {
        let ap: Message = serde_json::from_str(json_str).unwrap();

        match ap.state {
            State::Added => {log::info!("{} added to the list with {:?} {} ", ap.ssid, ap.field.unwrap(), ap.to.unwrap())},
            State::Changed => {log::info!("{}'s changed {:?} from {} to {}", ap.ssid, ap.field.unwrap(), ap.from.unwrap(), ap.to.unwrap())},
            State::Removed => {log::info!("{} is removed from the list", ap.ssid)},
        }
    } else {
        log::error!("Failed to create json from data transferred over pipe");
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut pipe = unix_named_pipe::open_read(PIPE_PATH).expect("could not open pipe for reading");
    loop {
        let mut payload: [u8; 128] = [0; 128];
        let res = pipe.read(&mut payload);
        if let Ok(count) = res {
            if count > 0 {
                let mut payload_str = String::new();
                for i in 0..count {
                    payload_str.push(payload[i] as char);
                }
                //log::info!("server count:{}", count);
                //log::info!("server str:{}", std::str::from_utf8(&payload).unwrap());
                parse_json_from_server(&payload_str);
            }
        }
    }
}