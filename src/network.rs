use std::fs;

#[derive(Debug)]
pub struct NetworkInfo {
    pub interface: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

pub fn get_network_info() -> std::io::Result<Vec<NetworkInfo>> {
    let content = fs::read_to_string("/proc/net/dev")?;
    let mut interfaces = Vec::new();

    for line in content.lines().skip(2) {
        let Some((interface, stats)) = line.split_once(':') else {
            continue;
        };

        let values: Vec<u64> = stats
            .split_whitespace()
            .filter_map(|v| v.parse().ok())
            .collect();

        if values.len() < 9 {
            continue;
        }

        interfaces.push(NetworkInfo {
            interface: interface.trim().to_string(),
            received_bytes: values[0],
            transmitted_bytes: values[8],
        });
    }

    Ok(interfaces)
}
