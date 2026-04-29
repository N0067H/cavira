use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
pub struct DetailSample {
    pub timestamp_ms: u64,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RunDetail {
    pub id: String,
    pub timestamp: u64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub peak_cpu: f32,
    pub avg_cpu: f32,
    pub peak_memory_bytes: u64,
    pub avg_memory_bytes: u64,
    pub samples: Vec<DetailSample>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: u64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    pub duration_ms: u64,
    pub peak_cpu: f32,
    pub avg_cpu: f32,
    pub peak_memory_bytes: u64,
    pub avg_memory_bytes: u64,
}

fn data_base() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local").join("share")
        })
        .join("cavira")
}

fn runs_dir() -> PathBuf {
    data_base().join("runs")
}

pub fn save_run(detail: &RunDetail) {
    let dir = runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(detail) {
        let _ = std::fs::write(dir.join(format!("{}.json", detail.id)), json);
    }
}

pub fn load_run(id: &str) -> Option<RunDetail> {
    let dir = runs_dir();
    let exact = dir.join(format!("{}.json", id));
    if exact.exists() {
        let content = std::fs::read_to_string(&exact).ok()?;
        return serde_json::from_str(&content).ok();
    }
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(id) && name_str.ends_with(".json") {
            let content = std::fs::read_to_string(entry.path()).ok()?;
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

fn history_path() -> PathBuf {
    data_base().join("history.json")
}

pub fn load() -> Vec<HistoryEntry> {
    let path = history_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn append(entry: HistoryEntry) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut entries = load();
    entries.push(entry);
    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
