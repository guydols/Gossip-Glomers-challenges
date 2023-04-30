use serde_json::Map;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

pub fn convert_map(map: Map<String, Value>) -> BTreeMap<String, Vec<String>> {
    let mut btree_map = BTreeMap::new();
    for (key, value) in map.iter() {
        if let Value::Array(array) = value {
            let vec = array
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            btree_map.insert(key.clone(), vec);
        }
    }
    btree_map
}

pub fn compare_knowledge(local: Vec<usize>, remote: Vec<usize>) -> Vec<usize> {
    let mut result: Vec<usize> = vec![];
    for x in local{
        if !remote.contains(&x) {
            result.push(x);
        }
    }
    result
}

pub fn log_string_to_file(s: &str) -> Res<()> {
    // Get the path of the executable
    let exe_path = std::env::current_exe()?;

    // Create a new path by replacing the executable's filename with "log.txt"
    let mut log_path = PathBuf::new();
    log_path.push(exe_path.parent().unwrap()); // Push the executable's parent directory
    log_path.push("debugging_log.txt"); // Push the log filename

    // Open the log file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    file.write_all(s.as_bytes())?;
    file.write_all(b"\n")?;

    Ok(())
}
