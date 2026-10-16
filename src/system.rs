use std::fs;
use std::io;

#[derive(Debug)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel: String,
    pub uptime_seconds: f64,
}

pub fn get_system_info() -> io::Result<SystemInfo> {
    let hostname = fs::read_to_string("/proc/sys/kernel/hostname")?
        .trim()
        .to_string();

    let kernel = fs::read_to_string("/proc/sys/kernel/osrelease")?
        .trim()
        .to_string();

    let uptime = fs::read_to_string("/proc/uptime")?;

    let uptime_seconds = uptime
        .split_whitespace()
        .next()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(SystemInfo {
        hostname,
        kernel,
        uptime_seconds,
    })
}
