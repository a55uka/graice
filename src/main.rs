use std::fs::File;
use std::{env, io, process};
use std::io::{Read, Write};
use std::path::Path;
use graice::emulator::{EmulatorConfig, N64Emulator};
use graice::rom::RomLoader;

// .n64 = Little Endian
// .v64 Byte Swapped
// .z64 Big Endian

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        eprintln!("Supported formats: .z64, .v64, .n64");
        process::exit(1);
    }

    let rom_path = &args[1];
    let rom_path = Path::new(rom_path);

    let mut file = File::open(rom_path)?;
    let mut rom_data = Vec::new();
    file.read_to_end(&mut rom_data)?;

    if let Err(e) = RomLoader::validate_file(rom_path) {
        eprintln!("ROM validation failed: {}", e);
        process::exit(1);
    }

    let rom = match RomLoader::load(rom_path) {
        Ok(rom) => rom,
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            process::exit(1);
        }
    };

    rom.print_info();

    if !rom.is_standard() {
        eprintln!("WARNING: ROM is unstandard");
    }

    let config = EmulatorConfig {
        debug: true,
        max_instructions: 100, // Limit for testing
        timeout: 5000000,     // 5 second timeout
    };

    let mut emulator = match N64Emulator::new(config) {
        Ok(emu) => emu,
        Err(e) => {
            eprintln!("Failed to create emulator: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = emulator.initialize_memory() {
        eprintln!("Failed to initialize memory: {}", e);
        process::exit(1);
    }

    if let Err(e) = emulator.load_rom(&rom) {
        eprintln!("Failed to load ROM into emulator: {}", e);
        process::exit(1);
    }

    println!("\nStarting emulation...");
    match emulator.start_emulation(&rom) {
        Ok(result) => {
            println!("Emulation finished in {:?}", result.execution_time());
            if let Some(error) = result.error() {
                eprintln!("Emulation error: {}", error);
            }
            if let Err(e) = emulator.print_registers() {
                eprintln!("Failed to read registers: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Emulation failed: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}
