use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: String,
    pub memory_kb: u64,
}

/// Récupère la liste des processus Linux.
pub fn get_processes() -> io::Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    for entry in fs::read_dir("/proc")? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let filename = entry.file_name();
        let Some(pid_str) = filename.to_str() else {
            continue;
        };

        // Dans /proc, les dossiers des processus portent leur PID.
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };

        let process_path = entry.path();

        let Ok(stat) = fs::read_to_string(process_path.join("stat")) else {
            continue;
        };

        let Some((name, state)) = parse_stat(&stat) else {
            continue;
        };

        let memory_kb = read_memory_kb(&process_path);

        processes.push(ProcessInfo {
            pid,
            name,
            state,
            memory_kb,
        });
    }

    Ok(processes)
}

/// Analyse /proc/<PID>/stat.
fn parse_stat(stat: &str) -> Option<(String, String)> {
    // Le nom peut contenir des espaces et des parenthèses.
    let start = stat.find('(')?;
    let end = stat.rfind(')')?;

    let name = stat[start + 1..end].to_string();

    // Après le dernier ')', les champs commencent par l'état (champ 3).
    let fields: Vec<&str> = stat[end + 1..].split_whitespace().collect();

    let state = fields.first()?.to_string();

    Some((name, state))
}

/// Lit la mémoire résidente d'un processus.
fn read_memory_kb(process_path: &Path) -> u64 {
    let Ok(status) = fs::read_to_string(process_path.join("status")) else {
        return 0;
    };

    for line in status.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            return value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
        }
    }

    0
}
