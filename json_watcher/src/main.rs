use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::env;
use notify::event::{AccessKind, AccessMode};
use serde_json::Value;
use std::fs;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    if let Err(error) = watch(path) {
        log::error!("Error: {error:?}");
    }
}

fn parse_json(path: PathBuf) {
    let json = {
        let file_content = fs::read_to_string(path.display().to_string().clone()).expect("LogRocket: error reading file");
        serde_json::from_str::<Value>(&file_content)
    };
    if json.is_ok() {
        let access_points = &json.unwrap()["access_points"];
        let ap_array = access_points.as_array();
        
        if ap_array.is_some() {
            let size = ap_array.unwrap().len();
            for i in 0..size {
                let ap = &ap_array.unwrap()[i];
                log::info!("ssid: {} snr: {} channel: {}", ap["ssid"], ap["snr"], ap["channel"]);
            }
        } else {
            log::error!("No field 'access_points' in json file");
        }
    } else {
        log::error!("parsing error in json file");
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let dir = env::current_dir().unwrap();
    let mut full_path = PathBuf::from(dir.clone());
    full_path.push(path);
    
    log::info!("Current dir: {} ", dir.display().to_string());
    log::info!("Full path: {} ", full_path.display().to_string());
    watcher.watch(&dir, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if event.paths[0] == full_path && event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)){
                    log::info!("changed file: {}", full_path.display().to_string());
                    parse_json(full_path.clone());
                };
                
            },
            Err(error) => log::error!("Error: {error:?}"),
        };
    }

    Ok(())
}