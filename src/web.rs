use std::{env, error::Error, net::SocketAddr};

use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::{battery, cpu, disk, memory, network, processes, system, temperature};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let address = env::var("TOPRS_WEB_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse::<SocketAddr>()?;
    let listener = TcpListener::bind(address).await?;
    let public_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/public");
    let app = Router::new()
        .route("/api/system", get(system_snapshot))
        .fallback_service(ServeDir::new(public_dir).append_index_html_on_directories(true));

    println!("Serveur TOPRS disponible sur http://{address}");
    println!("Fichiers statiques servis depuis {public_dir}");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn system_snapshot() -> Result<Json<Value>, (StatusCode, String)> {
    collect_system_snapshot()
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

fn collect_system_snapshot() -> Result<Value, Box<dyn Error>> {
    let cpu = cpu::get_cpu_info()?;
    let memory = memory::get_memory_info()?;
    let disk = disk::get_disk_info("/")?;
    let batteries = battery::get_battery_info()?;
    let network = network::get_network_info()?;
    let mut processes = processes::get_processes()?;
    let system = system::get_system_info()?;
    let temperatures = temperature::get_temperatures()?;

    processes.sort_by(|left, right| right.memory_kb.cmp(&left.memory_kb));
    let process_count = processes.len();
    processes.truncate(100);

    Ok(json!({
        "system": {
            "hostname": system.hostname,
            "kernel": system.kernel,
            "uptime_seconds": system.uptime_seconds,
        },
        "cpu": {
            "model": cpu.model,
            "logical_cores": cpu.logical_cores,
        },
        "memory": {
            "total_kb": memory.total,
            "available_kb": memory.available,
            "used_kb": memory.used,
            "usage_percent": memory.usage_percent,
            "swap_total_kb": memory.swap_total,
            "swap_used_kb": memory.swap_used,
        },
        "disk": {
            "total_bytes": disk.total_bytes,
            "free_bytes": disk.free_bytes,
            "available_bytes": disk.available_bytes,
            "used_bytes": disk.used_bytes,
        },
        "batteries": batteries.iter().map(|battery| json!({
            "name": battery.name,
            "capacity": battery.capacity,
            "status": battery.status,
        })).collect::<Vec<_>>(),
        "network": network.iter().map(|interface| json!({
            "interface": interface.interface,
            "received_bytes": interface.received_bytes,
            "transmitted_bytes": interface.transmitted_bytes,
        })).collect::<Vec<_>>(),
        "temperatures": temperatures.iter().map(|sensor| json!({
            "sensor": sensor.sensor,
            "temperature_celsius": sensor.temperature_celsius,
        })).collect::<Vec<_>>(),
        "process_count": process_count,
        "processes": processes.iter().map(|process| json!({
            "pid": process.pid,
            "name": process.name,
            "state": process.state,
            "memory_kb": process.memory_kb,
        })).collect::<Vec<_>>(),
    }))
}
