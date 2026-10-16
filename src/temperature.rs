use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct TemperatureInfo {
    pub sensor: String,
    pub temperature_celsius: f64,
}

pub fn get_temperatures() -> std::io::Result<Vec<TemperatureInfo>> {
    let root = Path::new("/sys/class/thermal");
    let mut temperatures = Vec::new();

    if !root.exists() {
        return Ok(temperatures);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        let name = entry.file_name().to_string_lossy().into_owned();

        if !name.starts_with("thermal_zone") {
            continue;
        }

        let Ok(raw) = fs::read_to_string(path.join("temp")) else {
            continue;
        };

        let Ok(value) = raw.trim().parse::<f64>() else {
            continue;
        };

        temperatures.push(TemperatureInfo {
            sensor: name,
            temperature_celsius: value / 1000.0,
        });
    }

    Ok(temperatures)
}
