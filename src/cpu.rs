use std::fs;

#[derive(Debug)]
pub struct CpuInfo {
    pub model: String,
    pub logical_cores: usize,
}

pub fn get_cpu_info() -> std::io::Result<CpuInfo> {
    let content = fs::read_to_string("/proc/cpuinfo")?;

    let mut model = String::from("Inconnu");
    let mut logical_cores = 0;

    for line in content.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "model name" if model == "Inconnu" => {
                    model = value.trim().to_string();
                }
                "processor" => logical_cores += 1,
                _ => {}
            }
        }
    }

    Ok(CpuInfo {
        model,
        logical_cores,
    })
}