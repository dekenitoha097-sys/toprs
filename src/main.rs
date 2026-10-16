mod battery;
mod cpu;

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

    Ok(())
}
