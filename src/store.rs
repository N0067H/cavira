use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
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

fn history_jsonl_path() -> PathBuf {
    data_base().join("history.jsonl")
}

pub fn load() -> Vec<HistoryEntry> {
    load_history(&history_path(), &history_jsonl_path())
}

pub fn append(entry: HistoryEntry) {
    let path = history_jsonl_path();
    append_history(&path, &entry);
}

fn append_history(path: &Path, entry: &HistoryEntry) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = serde_json::to_writer(&mut file, &entry);
        let _ = writeln!(file);
    }
}

fn load_history(legacy_path: &Path, jsonl_path: &Path) -> Vec<HistoryEntry> {
    let mut entries = load_legacy_history(legacy_path);
    entries.extend(load_jsonl_history(jsonl_path));
    entries
}

fn load_legacy_history(path: &Path) -> Vec<HistoryEntry> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![];
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn load_jsonl_history(path: &Path) -> Vec<HistoryEntry> {
    let Ok(file) = File::open(path) else {
        return vec![];
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            timestamp: 1,
            source: "run".to_string(),
            command: Some(format!("echo {id}")),
            pid: None,
            process_name: None,
            duration_ms: 10,
            peak_cpu: 1.0,
            avg_cpu: 0.5,
            peak_memory_bytes: 1024,
            avg_memory_bytes: 512,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cavira-store-test-{}-{}-{}",
            std::process::id(),
            now_ms(),
            name
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_history_reads_legacy_json_and_jsonl_in_order() {
        let dir = temp_dir("load");
        let legacy_path = dir.join("history.json");
        let jsonl_path = dir.join("history.jsonl");

        std::fs::write(
            &legacy_path,
            serde_json::to_string(&vec![entry("legacy-1"), entry("legacy-2")]).unwrap(),
        )
        .unwrap();

        let mut jsonl = File::create(&jsonl_path).unwrap();
        serde_json::to_writer(&mut jsonl, &entry("jsonl-1")).unwrap();
        writeln!(jsonl).unwrap();
        serde_json::to_writer(&mut jsonl, &entry("jsonl-2")).unwrap();
        writeln!(jsonl).unwrap();

        let entries = load_history(&legacy_path, &jsonl_path);
        let ids: Vec<_> = entries.into_iter().map(|entry| entry.id).collect();

        assert_eq!(ids, ["legacy-1", "legacy-2", "jsonl-1", "jsonl-2"]);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn append_history_writes_one_jsonl_record_without_touching_legacy_json() {
        let dir = temp_dir("append");
        let legacy_path = dir.join("history.json");
        let jsonl_path = dir.join("history.jsonl");
        let legacy_content = serde_json::to_string_pretty(&vec![entry("legacy")]).unwrap();
        std::fs::write(&legacy_path, &legacy_content).unwrap();

        if let Some(parent) = jsonl_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        append_history(&jsonl_path, &entry("new"));

        assert_eq!(
            std::fs::read_to_string(&legacy_path).unwrap(),
            legacy_content
        );

        let entries = load_history(&legacy_path, &jsonl_path);
        let ids: Vec<_> = entries.into_iter().map(|entry| entry.id).collect();
        assert_eq!(ids, ["legacy", "new"]);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
