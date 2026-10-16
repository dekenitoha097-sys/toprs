use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct BatteryInfo {
    pub name: String,
    pub capacity: Option<u8>,
    pub status: Option<String>,
}

pub fn get_battery_info() -> std::io::Result<Vec<BatteryInfo>> {
    let root = Path::new("/sys/class/power_supply");
    let mut batteries = Vec::new();

    if !root.exists() {
        return Ok(batteries);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        let battery_type = fs::read_to_string(path.join("type")).unwrap_or_default();

        if battery_type.trim() != "Battery" {
            continue;
        }

        let capacity = fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|v| v.trim().parse::<u8>().ok());

        let status = fs::read_to_string(path.join("status"))
            .ok()
            .map(|v| v.trim().to_string());

        batteries.push(BatteryInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            capacity,
            status,
        });
    }

    Ok(batteries)
}
