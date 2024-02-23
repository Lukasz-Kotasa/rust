use hash_map_diff::hash_map_diff;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{AccessKind, AccessMode};
use serde_json::Value;
use std::collections::HashMap;
use std::{env, fs, thread};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

//from lib.rs
use json_watcher::PIPE_PATH;
use json_watcher::{State, Field, Message};
use notify::Event;

extern crate unix_named_pipe;
extern crate ctrlc;

#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]
struct AP {
    snr: u64,
    channel: u64,
}

impl AP {
    pub fn new(snr: u64, channel: u64) -> Self { AP {snr,channel,} }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
   
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    // Create new named pipe
    let mode = 0o644;
    let _res = unix_named_pipe::create(PIPE_PATH, Some(mode)); 

    if let Err(error) = watch(path) {
            log::error!("Error: {error:?}");
    }

    let _ = fs::remove_file(PIPE_PATH);
}

fn json_to_hasmap(path: PathBuf) -> HashMap<String, AP> {
    let json = {
        let file_content = fs::read_to_string(path.display().to_string().clone()).expect("Error reading json file");
        serde_json::from_str::<Value>(&file_content)
    };
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

fn send_to_pipe(message: &Message) {
    let mut pipe = unix_named_pipe::open_write(PIPE_PATH).expect("could not open pipe for writing");
    let serialized_msg =  serde_json::to_string(&message).unwrap_or_else(|error| panic!("Could not serialize Message, error: {:?}", error));
    pipe.write(serialized_msg.as_bytes()).expect("could not write payload to pipe");
}

fn find_changes(old: &HashMap<String, AP>, new: &HashMap<String, AP>) {
    let received_diff = hash_map_diff(&old, &new);

    let removed: HashMap<&String, &AP> = received_diff.removed;
    let updated: HashMap<&String, &AP> = received_diff.updated;


    for (ssid, ap) in updated {
        let old_ap = old.get_key_value(ssid);
        // check if this modified AP is existing or new one
        if old_ap.is_none() {
            // new one
            send_to_pipe(&Message{state: State::Added,
                                  ssid: ssid.clone(),
                                  field: Some(Field::Snr),
                                  from: Some(0),
                                  to: Some(ap.snr)});
            

            send_to_pipe(&Message{state: State::Added,
                                  ssid: ssid.clone(),
                                  field: Some(Field::Channel),
                                  from: Some(0),
                                  to: Some(ap.channel)});
            log::info!("Added ssid: {}, anr: {}, ch: {}", ssid, ap.snr, ap.channel);
        } else {
            // existing one
            let old_snr = old_ap.unwrap().1.snr;
            if old_snr != ap.snr {
                send_to_pipe(&Message{state: State::Changed,
                                      ssid: ssid.clone(),
                                      field: Some(Field::Snr),
                                      from: Some(old_snr),
                                      to: Some(ap.snr)});
                log::info!("Changed ssid: {}, snr: from {}, to: {}", ssid, old_snr, ap.snr);
            }
            let old_channel = old_ap.unwrap().1.channel;
            if old_channel != ap.channel {
                send_to_pipe(&Message{state: State::Changed,
                                      ssid: ssid.clone(),
                                      field: Some(Field::Channel),
                                      from: Some(old_channel),
                                      to: Some(ap.channel)});
                log::info!("Changed ssid: {}, channel: from {}, to: {}", ssid, old_channel, ap.channel);
            }
        }
    }

    for (ssid, _ap) in removed {
        send_to_pipe(&Message{state: State::Removed,
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

    return running;
}


fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx.clone(), Config::default())?;

    let dir = env::current_dir().unwrap();
    let mut full_path = PathBuf::from(dir.clone());
    full_path.push(path);

    watcher.watch(&dir, RecursiveMode::Recursive)?;

    let mut old_hash: HashMap<String, AP> = json_to_hasmap(full_path.clone());

    let handle = thread::spawn(move || {
        log::info!("monitoring thread started");
        for res in rx {
            match res {
                Ok(event) => {
                    if event.kind == EventKind::Any {
                        log::info!("monitoring thread shutting down");
                        break;
                    }  else if event.paths[0] == full_path && event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)){
                        let new_hash: HashMap<String, AP> = json_to_hasmap(full_path.clone());
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