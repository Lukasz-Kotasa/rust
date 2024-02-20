use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::env;
use notify::event::{AccessKind, AccessMode};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    if let Err(error) = watch(path) {
        log::error!("Error: {error:?}");
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
                if event.paths[0] == full_path  && event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)){
                    log::info!("changed file: {}", full_path.display());
                }
            },
            Err(error) => log::error!("Error: {error:?}"),
        };
    }

    Ok(())
}