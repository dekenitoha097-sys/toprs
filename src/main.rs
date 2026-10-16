mod battery;
mod cpu;
mod disk;
mod network;
mod temperature;

fn main() -> std::io::Result<()> {
    let cpu = cpu::get_cpu_info()?;
    println!("Informations CPU");
    println!("Modèle : {}", cpu.model);
    println!("Cœurs logiques : {}", cpu.logical_cores);

    println!("\nInformations batterie");
    let batteries = battery::get_battery_info()?;

    if batteries.is_empty() {
        println!("Aucune batterie détectée.");
    } else {
        for battery in batteries {
            println!("Nom : {}", battery.name);
            println!(
                "Charge : {}",
                battery.capacity.map_or_else(
                    || "indisponible".to_string(),
                    |capacity| format!("{capacity}%")
                )
            );
            println!(
                "État : {}",
                battery.status.as_deref().unwrap_or("indisponible")
            );
        }
    }

    let disk = disk::get_disk_info("/")?;
    let gib = 1024.0 * 1024.0 * 1024.0;
    println!("\nInformations disque (/)");
    println!("Utilisé : {:.2} Gio", disk.used_bytes as f64 / gib);
    println!("Libre : {:.2} Gio", disk.free_bytes as f64 / gib);
    println!("Disponible : {:.2} Gio", disk.available_bytes as f64 / gib);
    println!("Total : {:.2} Gio", disk.total_bytes as f64 / gib);

    let interfaces = network::get_network_info()?;
    println!("\nInformations réseau");
    if interfaces.is_empty() {
        println!("Aucune interface réseau détectée.");
    } else {
        for interface in interfaces {
            println!("Interface : {}", interface.interface);
            println!("Reçus : {} octets", interface.received_bytes);
            println!("Transmis : {} octets", interface.transmitted_bytes);
        }
    }

    let temperatures = temperature::get_temperatures()?;
    println!("\nTempératures");
    if temperatures.is_empty() {
        println!("Aucun capteur thermique détecté.");
    } else {
        for temperature in temperatures {
            println!(
                "{} : {:.1} °C",
                temperature.sensor, temperature.temperature_celsius
            );
        }
    }

    Ok(())
}
