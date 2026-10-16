use nix::sys::statvfs::statvfs;
use std::io;

#[derive(Debug)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
}

pub fn get_disk_info(path: &str) -> io::Result<DiskInfo> {
    let stats = statvfs(path).map_err(|e| io::Error::other(e.to_string()))?;

    let block_size = stats.fragment_size() as u64;
    let total_bytes = stats.blocks() * block_size;
    let free_bytes = stats.blocks_free() * block_size;
    let available_bytes = stats.blocks_available() * block_size;

    Ok(DiskInfo {
        total_bytes,
        free_bytes,
        available_bytes,
        used_bytes: total_bytes.saturating_sub(free_bytes),
    })
}
