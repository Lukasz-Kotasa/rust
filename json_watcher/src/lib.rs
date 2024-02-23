use serde::{Serialize, Deserialize};

pub const PIPE_PATH: &str = "/tmp/json_watcher.pipe";

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    Added,
    Changed,
    Removed,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Field {
    Snr,
    Channel,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub state: State,
    pub ssid: String,
    pub field: Option<Field>,
    pub from: Option<u64>,
    pub to: Option<u64>,
}