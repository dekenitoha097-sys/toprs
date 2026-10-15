use std::fs;

#[derive(Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub usage_percent: f64,
}

pub fn get_memory_info() -> std::io::Result<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo")?;

    let mut total = 0;
    let mut available = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;

    for line in content.lines() {
        let mut parts = line.split_whitespace();

        let key = parts.next().unwrap_or("");
        let value = parts
            .next()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        match key {
            "MemTotal:" => total = value,
            "MemAvailable:" => available = value,
            "SwapTotal:" => swap_total = value,
            "SwapFree:" => swap_free = value,
            _ => {}
        }
    }

    let used = total.saturating_sub(available);
    let swap_used = swap_total.saturating_sub(swap_free);

    let usage_percent = if total > 0 {
        used as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    Ok(MemoryInfo {
        total,
        available,
        used,
        swap_total,
        swap_used,
        usage_percent,
    })
}