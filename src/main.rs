mod battery;
mod cpu;
mod disk;

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

    Ok(())
}
