mod cpu;

fn main() -> std::io::Result<()> {
    let cpu = cpu::get_cpu_info()?;

    println!("Informations CPU");
    println!("Modèle : {}", cpu.model);
    println!("Cœurs logiques : {}", cpu.logical_cores);

    Ok(())
}