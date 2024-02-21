use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use notify::event::{AccessKind, AccessMode};
use serde_json::Value;
use std::fs;
use hash_map_diff::hash_map_diff;

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

    if let Err(error) = watch(path) {
        log::error!("Error: {error:?}");
    }
}

fn json_to_hasmap(path: PathBuf) -> HashMap<String, AP> {
    let json = {
        let file_content = fs::read_to_string(path.display().to_string().clone()).expect("LogRocket: error reading file");
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

fn find_changes(old: &HashMap<String, AP>, new: &HashMap<String, AP>) {
    let received_diff = hash_map_diff(&old, &new);

    let removed = received_diff.removed;
    let updated = received_diff.updated;

    for (ssid, ap) in updated {
        let old_ap = old.get_key_value(ssid);
        if old_ap.is_none() {
            log::info!("Added ssid: {}, anr: {}, ch: {}", ssid, ap.snr, ap.channel);
        } else {
            if old_ap.unwrap().1.snr != ap.snr {
                log::info!("Changes in ssid {} snr to: {}", ssid, ap.snr);
            }
            if old_ap.unwrap().1.channel != ap.channel {
                log::info!("Changes in ssid {} channel to: {}", ssid, ap.channel);
            }
        }
    }

    for (ssid, _ap) in removed {
        log::info!("Removed ssid: {}", ssid);
    }
    
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let dir = env::current_dir().unwrap();
    let mut full_path = PathBuf::from(dir.clone());
    full_path.push(path);
    
    //log::info!("Current dir: {} ", dir.display().to_string());
    //log::info!("Full path: {} ", full_path.display().to_string());

    watcher.watch(&dir, RecursiveMode::Recursive)?;

    let mut old_hash: HashMap<String, AP> = json_to_hasmap(full_path.clone());

    for res in rx {
        match res {
            Ok(event) => {
                if event.paths[0] == full_path && event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)){
                    log::info!("changed file: {}", full_path.display().to_string());
                    let new_hash = json_to_hasmap(full_path.clone());
                    find_changes(&old_hash, &new_hash);
                    old_hash = new_hash;
                };
                
            },
            Err(error) => log::error!("Error: {error:?}"),
        };
    }

    Ok(())
}