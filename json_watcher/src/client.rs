use std::io::Read;
use serde_json::Value;

//from lib.rs
use json_watcher::PIPE_PATH;
use json_watcher::{State, Message};

extern crate unix_named_pipe;

fn parse_json_from_server(json_str: &str) -> String {
    let json: Result<Value, serde_json::Error> = serde_json::from_str::<Value>(json_str);

    if json.is_ok() {
        let ap: Message = serde_json::from_str(json_str).unwrap();
        match ap.state {
            State::Added => format!("{} added to the list with {:?} {}", ap.ssid, ap.field.unwrap(), ap.to.unwrap()),
            State::Changed => format!("{}'s changed {:?} from {} to {}", ap.ssid, ap.field.unwrap(), ap.from.unwrap(), ap.to.unwrap()),
            State::Removed => format!("{} is removed from the list", ap.ssid),
        }
    } else {
        format!("Failed to create json from data transferred over pipe")
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut pipe: std::fs::File = unix_named_pipe::open_read(PIPE_PATH).expect("could not open pipe for reading");
    loop {
        let mut payload: [u8; 128] = [0; 128];
        let res: Result<usize, std::io::Error> = pipe.read(&mut payload);
        if let Ok(count) = res {
            if count > 0 {
                let mut payload_str: String = String::new();
                for i in 0..count {
                    payload_str.push(payload[i] as char);
                }
                let message = parse_json_from_server(&payload_str);
                println!("{}", message);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::parse_json_from_server;
    #[test]
    fn parsing_server_messages_changed() {
        assert_eq!(parse_json_from_server(r#"{"state":"Changed","ssid":"HerAP","field":"Snr","from":8,"to":1}"#), "HerAP's changed Snr from 8 to 1".to_string());
        assert_eq!(parse_json_from_server(r#"{"state":"Changed","ssid":"HerAP","field":"Channel","from":1,"to":0}"#), "HerAP's changed Channel from 1 to 0".to_string());
    }
    #[test]
    fn parsing_server_messages_added() {
        assert_eq!(parse_json_from_server(r#"{"state":"Added","ssid":"HerAP","field":"Snr","from":0,"to":1}"#), "HerAP added to the list with Snr 1".to_string());
    }
    #[test]
    fn parsing_server_messages_removed() {
        assert_eq!(parse_json_from_server(r#"{"state":"Removed","ssid":"HerAP","field":null,"from":null,"to":null}"#), "HerAP is removed from the list".to_string());
    }
}