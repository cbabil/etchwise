//! Lists the boot partition of a disk image and prints small text files:
//! `cargo run -p imager-core --example bootls -- <image> [file…]`.

use std::fs::File;

use imager_core::fat::BootPartition;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(image) = args.first() else {
        eprintln!("usage: bootls <image> [file…]");
        std::process::exit(2);
    };
    let mut boot = BootPartition::open(File::open(image)?)?;
    for file in boot.list_files()? {
        println!("{file}");
    }
    for name in &args[1..] {
        let data = boot.read_file(name)?.unwrap_or_default();
        println!("--- {name} ---\n{}", String::from_utf8_lossy(&data));
    }
    Ok(())
}
