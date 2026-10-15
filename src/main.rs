mod memory;

fn main() {
    let memory = memory::get_memory_info();
    println!("{}", memory.unwrap().swap_total);
}
