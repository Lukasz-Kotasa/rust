use hash_map_diff::hash_map_diff;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{AccessKind, AccessMode};
use serde_json::Value;
use std::collections::HashMap;
use std::{env, fs, thread};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use notify::Event;

//from lib.rs
use json_watcher::PIPE_PATH;
use json_watcher::{State, Field, Message};


extern crate unix_named_pipe;
extern crate ctrlc;

#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Hash)]
struct AP {
    snr: u64,
    channel: u64,
}

impl AP {
    pub fn new(snr: u64, channel: u64) -> Self { AP {snr,channel,} }
}

#[derive(Debug)]
struct ServerError {
    kind: String,
    message: String,
}

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> Self {
        ServerError {
            kind: String::from("io"),
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(error: serde_json::Error) -> Self {
        ServerError {
            kind: String::from("serdes"),
            message: error.to_string(),
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
   
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    // Create new named pipe
    let mode: u32 = 0o644;
    let res: Result<(), std::io::Error> = unix_named_pipe::create(PIPE_PATH, Some(mode)); 
    if res.is_err() {
        log::error!("Could not create named pipe: {}", res.err().unwrap());
        return;
    }

    if let Err(error) = watch(path) {
            log::error!("Error: {error:?}");
    }

    let res: Result<(), std::io::Error> = fs::remove_file(PIPE_PATH);
    if res.is_err() {
        log::error!("Could not remove named pipe: {}", res.err().unwrap());
        return;
    }
}

fn json_to_hasmap(file_content: String) -> HashMap<String, AP> {
    let json = serde_json::from_str::<Value>(&file_content);
    let mut ap_hash = HashMap::new();
    
    if json.is_ok() {
        let access_points = &json.unwrap()["access_points"];
        let ap_array = access_points.as_array();

        if ap_array.is_some() {
            let size = ap_array.unwrap().len();
            
            for i in 0..size {
                let ap = &ap_array.unwrap()[i];
                ap_hash.insert(ap["ssid"].as_str().unwrap().to_string(), AP::new(ap["snr"].as_u64().unwrap(), ap["channel"].as_u64().unwrap()));
            }
        } else {
            log::error!("No field 'access_points' in json file");
        }
    } else {
        log::error!("Parsing error in json file");
    }
    ap_hash
}

fn send_to_pipe(message: &Message) -> Result<(), ServerError> {
    // give some time to the client to read previous data from the pipe
    thread::sleep(Duration::from_millis(50));

    let pipe_res  = unix_named_pipe::open_write(PIPE_PATH);
    let mut pipe = match pipe_res {
        Ok(file) => file,
        Err(error) => return Err(error.into()),
    };

    let serialized_msg_res = serde_json::to_string(&message);
    let serialized_msg = match serialized_msg_res {
        Ok(msg) => msg,
        Err(error) => return Err(error.into()),
    };

    let write_res = pipe.write(serialized_msg.as_bytes());
    match write_res {
        Ok(size) => log::info!("succ wrote {} bytes to a pipe", size),
        Err(error) => return Err(error.into()),
    };

    Ok(())
}

fn find_changes(old: &HashMap<String, AP>, new: &HashMap<String, AP>) {
    let received_diff: hash_map_diff::HashMapDiff<&String, &AP> = hash_map_diff(&old, &new);

    let removed: HashMap<&String, &AP> = received_diff.removed;
    let updated: HashMap<&String, &AP> = received_diff.updated;

    for (ssid, ap) in updated {
        let old_ap = old.get_key_value(ssid);
        // check if this modified AP is existing or new one
        if old_ap.is_none() {
            // new one
            let _ = send_to_pipe(&Message{state: State::Added,
                                  ssid: ssid.clone(),
                                  field: Some(Field::Snr),
                                  from: Some(0),
                                  to: Some(ap.snr)});
            

            let _ = send_to_pipe(&Message{state: State::Added,
                                  ssid: ssid.clone(),
                                  field: Some(Field::Channel),
                                  from: Some(0),
                                  to: Some(ap.channel)});
            log::info!("Added ssid: {}, snr: {}, ch: {}", ssid, ap.snr, ap.channel);
        } else {
            // existing one
            let old_snr = old_ap.unwrap().1.snr;
            if old_snr != ap.snr {
                let _ = send_to_pipe(&Message{state: State::Changed,
                                      ssid: ssid.clone(),
                                      field: Some(Field::Snr),
                                      from: Some(old_snr),
                                      to: Some(ap.snr)});
                log::info!("Changed ssid: {}, snr: from {}, to: {}", ssid, old_snr, ap.snr);
            }
            let old_channel = old_ap.unwrap().1.channel;
            if old_channel != ap.channel {
                let _ = send_to_pipe(&Message{state: State::Changed,
                                      ssid: ssid.clone(),
                                      field: Some(Field::Channel),
                                      from: Some(old_channel),
                                      to: Some(ap.channel)});
                log::info!("Changed ssid: {}, channel: from {}, to: {}", ssid, old_channel, ap.channel);
            }
        }
    }

    for (ssid, _ap) in removed {
        let _ = send_to_pipe(&Message{state: State::Removed,
                              ssid: ssid.clone(),
                              field: None,
                              from: None,
                              to: None});
        log::info!("Removed ssid: {}", ssid);
    }
    
}

fn make_loop_flag() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("could not set up keyboard interrupt handler");

    running
}


fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx.clone(), Config::default())?;

    let dir = env::current_dir().unwrap();
    let mut full_path = PathBuf::from(dir.clone());
    full_path.push(path);

    watcher.watch(&dir, RecursiveMode::Recursive)?;

    let json_string_res = fs::read_to_string(full_path.clone()); //.expect("Error reading json file");
    let json_string = match json_string_res {
        Ok(str) => str,
        Err(error) => return Err(error.into()),
    };
    let mut old_hash: HashMap<String, AP> = json_to_hasmap(json_string);

    let handle = thread::spawn(move || {
        log::info!("monitoring thread started");
        for res in rx {
            match res {
                Ok(event) => {
                    if event.kind == EventKind::Any {
                        log::info!("monitoring thread shutting down");
                        break;
                    }  else if event.paths[0] == full_path && event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)){
                        let json_string = fs::read_to_string(full_path.clone()).expect("Error reading json file");
                        let new_hash: HashMap<String, AP> = json_to_hasmap(json_string);
                        find_changes(&old_hash, &new_hash);
                        old_hash = new_hash;
                    };
                },
                Err(error) => log::error!("Error: {error:?}"),
            };
        }
    });

    let running = make_loop_flag();

    while running.load(Ordering::SeqCst) {}
    
    log::info!("main thread sends close req to monitoring thread");
    let _ = tx.send(Ok(Event::new(EventKind::Any)));
    handle.join().unwrap();
    log::info!("main thread joined after monitoring thread finished");

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::json_to_hasmap;
    use crate::send_to_pipe;
    use crate::AP;
    use crate::State;
    use crate::Message;
    use std::collections::HashMap;
    #[test]
    fn converting_json_with_one_ap_to_hasmap() {
        assert_eq!(json_to_hasmap(r#"{ "access_points": [{ "ssid": "MyAP", "snr": 61, "channel": 1 }]}"#.to_string()),
                    HashMap::from([("MyAP".to_string(), AP::new(61,1))]));
    }
    #[test]
    fn converting_json_with_three_aps_to_hasmap() {
        assert_eq!(json_to_hasmap(r#"{"access_points": [ { "ssid": "MyAP", "snr": 1, "channel": 2 }, { "ssid": "YourAP", "snr": 3, "channel": 4 }, { "ssid": "HisAP", "snr": 5, "channel": 6}]}"#.to_string()),
        HashMap::from([("MyAP".to_string(), AP::new(1,2)),
                       ("YourAP".to_string(), AP::new(3,4)),
                       ("HisAP".to_string(), AP::new(5,6)),
        ]));
    }

    #[test]
    fn send_to_pipe_fails_with_no_pipe_created() {
        assert!(send_to_pipe(&Message{state: State::Removed,
            ssid: "SSID".to_string(),
            field: None,
            from: None,
            to: None}).is_err_and(|e| e.kind == "io"));
    }

}