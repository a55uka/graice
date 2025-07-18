use crate::error::{GraiceError, Result};
use crate::rom::N64Rom;
use unicorn_engine::{Arch, Mode, Permission, RegisterMIPS, Unicorn};

/// N64 Memory Map Constants
pub mod memory {
    /// RDRAM (Random Access Memory) base address
    pub const RDRAM_BASE: u64 = 0x0000_0000;
    /// RDRAM size (8MB)
    pub const RDRAM_SIZE: u64 = 0x0080_0000;

    /// Cartridge ROM base address
    pub const ROM_BASE: u64 = 0x1000_0000;
    /// Maximum ROM size (64MB)
    pub const ROM_SIZE: u64 = 0x0400_0000;

    /// SP DMEM (Data Memory) base address
    pub const SP_DMEM_BASE: u64 = 0x0400_0000;
    /// SP DMEM size (4KB)
    pub const SP_DMEM_SIZE: u64 = 0x0000_1000;

    /// SP IMEM (Instruction Memory) base address
    pub const SP_IMEM_BASE: u64 = 0x0400_1000;
    /// SP IMEM size (4KB)
    pub const SP_IMEM_SIZE: u64 = 0x0000_1000;

    /// PI (Peripheral Interface) registers base
    pub const PI_BASE: u64 = 0x0460_0000;
    /// PI registers size
    pub const PI_SIZE: u64 = 0x0000_1000;
}

/// N64 Emulator configuration
#[derive(Debug, Clone)]
pub struct EmulatorConfig {
    /// Enable debugging output
    pub debug: bool,
    /// Maximum number of instructions to execute (0 = unlimited)
    pub max_instructions: u64,
    /// Timeout in microseconds (0 = no timeout)
    pub timeout: u64,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            debug: false,
            max_instructions: 1000000, // Default limit to prevent infinite loops
            timeout: 1000000,         // 1 second timeout
        }
    }
}

/// N64 Emulator using Unicorn Engine
pub struct N64Emulator<'a> {
    engine: Unicorn<'a, ()>,
    config: EmulatorConfig,
}

impl<'a> N64Emulator<'a> {
    /// Create a new N64 emulator instance
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        let engine = Unicorn::new(Arch::MIPS, Mode::MIPS64 | Mode::BIG_ENDIAN).map_err(|e| {
            GraiceError::EmulationError(format!("Failed to initialize Unicorn engine: {:?}", e))
        })?;

        Ok(Self { engine, config })
    }

    /// Initialize memory maps for N64 emulation
    pub fn initialize_memory(&mut self) -> Result<()> {
        // RDRAM (read/write)
        self.engine
            .mem_map(
                memory::RDRAM_BASE,
                memory::RDRAM_SIZE as usize,
                Permission::READ | Permission::WRITE,
            )
            .map_err(|e| GraiceError::EmulationError(format!("Failed to map RDRAM: {:?}", e)))?;

        // ROM (read + execute)
        self.engine
            .mem_map(
                memory::ROM_BASE,
                memory::ROM_SIZE as usize,
                Permission::READ | Permission::EXEC,
            )
            .map_err(|e| GraiceError::EmulationError(format!("Failed to map ROM: {:?}", e)))?;

        // SP DMEM (read/write)
        self.engine
            .mem_map(
                memory::SP_DMEM_BASE,
                memory::SP_DMEM_SIZE as usize,
                Permission::READ | Permission::WRITE,
            )
            .map_err(|e| GraiceError::EmulationError(format!("Failed to map SP DMEM: {:?}", e)))?;

        // SP IMEM (read/write/execute)
        self.engine
            .mem_map(
                memory::SP_IMEM_BASE,
                memory::SP_IMEM_SIZE as usize,
                Permission::READ | Permission::WRITE | Permission::EXEC,
            )
            .map_err(|e| GraiceError::EmulationError(format!("Failed to map SP IMEM: {:?}", e)))?;

        if self.config.debug {
            println!("Memory maps initialized:");
            println!(
                "  RDRAM: 0x{:08x} - 0x{:08x}",
                memory::RDRAM_BASE,
                memory::RDRAM_BASE + memory::RDRAM_SIZE
            );
            println!(
                "  ROM:   0x{:08x} - 0x{:08x}",
                memory::ROM_BASE,
                memory::ROM_BASE + memory::ROM_SIZE
            );
            println!(
                "  SP DMEM: 0x{:08x} - 0x{:08x}",
                memory::SP_DMEM_BASE,
                memory::SP_DMEM_BASE + memory::SP_DMEM_SIZE
            );
            println!(
                "  SP IMEM: 0x{:08x} - 0x{:08x}",
                memory::SP_IMEM_BASE,
                memory::SP_IMEM_BASE + memory::SP_IMEM_SIZE
            );
        }

        Ok(())
    }

    /// Load ROM into emulator memory
    pub fn load_rom(&mut self, rom: &N64Rom) -> Result<()> {
        let rom_data = rom.full_data();

        if rom_data.len() > memory::ROM_SIZE as usize {
            return Err(GraiceError::EmulationError(
                "ROM too large for memory map".to_string(),
            ));
        }

        self.engine
            .mem_write(memory::ROM_BASE, rom_data)
            .map_err(|e| {
                GraiceError::EmulationError(format!("Failed to write ROM data: {:?}", e))
            })?;

        if self.config.debug {
            println!(
                "ROM loaded: {} bytes at 0x{:08x}",
                rom_data.len(),
                memory::ROM_BASE
            );
            println!("Boot point: 0x{:08x}", rom.header.boot_point);
        }

        Ok(())
    }

    /// Start emulation from ROM boot point
    pub fn start_emulation(&mut self, rom: &N64Rom) -> Result<EmulationResult> {
        let boot_address = self.calculate_boot_address(rom.header.boot_point)?;
        let end_address = memory::ROM_BASE + rom.rom_data().len() as u64;

        if self.config.debug {
            println!("Starting emulation:");
            println!("  Boot address: 0x{:08x}", boot_address);
            println!("  End address: 0x{:08x}", end_address);
        }

        let start_time = std::time::Instant::now();

        let result = self.engine.emu_start(
            boot_address,
            end_address,
            self.config.timeout,
            self.config.max_instructions as usize,
        );

        let execution_time = start_time.elapsed();

        match result {
            Ok(()) => {
                if self.config.debug {
                    println!("Emulation completed successfully in {:?}", execution_time);
                }
                Ok(EmulationResult::Success { execution_time })
            }
            Err(e) => {
                if self.config.debug {
                    println!("Emulation failed after {:?}: {:?}", execution_time, e);
                }
                Ok(EmulationResult::Error {
                    error: format!("Emulation error: {:?}", e),
                    execution_time,
                })
            }
        }
    }

    /// Calculate the actual boot address for emulation
    fn calculate_boot_address(&self, boot_point: u32) -> Result<u64> {
        // N64 boot point is usually 0x80000400
        // Map it to our ROM address space
        if boot_point >= 0x80000000 && boot_point <= 0x80000000 + memory::ROM_SIZE as u32 {
            // Virtual address in RDRAM space, map to ROM
            Ok(memory::ROM_BASE + (boot_point - 0x80000000) as u64)
        } else if boot_point >= 0x10000000 && boot_point <= 0x10000000 + memory::ROM_SIZE as u32 {
            // Physical ROM address
            Ok(boot_point as u64)
        } else {
            Err(GraiceError::EmulationError(format!(
                "Invalid boot point: 0x{:08x}",
                boot_point
            )))
        }
    }

    /// Read a register value
    pub fn read_register(&self, register: RegisterMIPS) -> Result<u64> {
        self.engine
            .reg_read(register)
            .map_err(|e| GraiceError::EmulationError(format!("Failed to read register: {:?}", e)))
    }

    /// Write a register value
    pub fn write_register(&mut self, register: RegisterMIPS, value: u64) -> Result<()> {
        self.engine
            .reg_write(register, value)
            .map_err(|e| GraiceError::EmulationError(format!("Failed to write register: {:?}", e)))
    }

    /// Read memory at the specified address
    pub fn read_memory(&self, address: u64, size: usize) -> Result<Vec<u8>> {
        self.engine
            .mem_read_as_vec(address, size)
            .map_err(|e| GraiceError::EmulationError(format!("Failed to read memory: {:?}", e)))
    }

    /// Write memory at the specified address
    pub fn write_memory(&mut self, address: u64, data: &[u8]) -> Result<()> {
        self.engine
            .mem_write(address, data)
            .map_err(|e| GraiceError::EmulationError(format!("Failed to write memory: {:?}", e)))
    }

    /// Get current configuration
    pub fn config(&self) -> &EmulatorConfig {
        &self.config
    }

    /// Update emulator configuration
    pub fn set_config(&mut self, config: EmulatorConfig) {
        self.config = config;
    }

    /// Print current register states (for debugging)
    pub fn print_registers(&self) -> Result<()> {
        let registers = [
            (RegisterMIPS::T0, "$t0"),
            (RegisterMIPS::T1, "$t1"),
            (RegisterMIPS::T2, "$t2"),
            (RegisterMIPS::T3, "$t3"),
            (RegisterMIPS::T4, "$t4"),
            (RegisterMIPS::T5, "$t5"),
            (RegisterMIPS::T6, "$t6"),
            (RegisterMIPS::T7, "$t7"),
            (RegisterMIPS::T8, "$t8"),
            (RegisterMIPS::T9, "$t9"),
            (RegisterMIPS::PC, "$pc"),
            (RegisterMIPS::SP, "$sp"),
            (RegisterMIPS::RA, "$ra"),
        ];

        println!("Register states:");
        for (reg, name) in registers {
            let value = self.read_register(reg)?;
            println!("  {}: 0x{:016x} ({})", name, value, value);
        }

        Ok(())
    }
}

/// Result of emulation execution
#[derive(Debug)]
pub enum EmulationResult {
    Success {
        execution_time: std::time::Duration,
    },
    Error {
        error: String,
        execution_time: std::time::Duration,
    },
}

impl EmulationResult {
    /// Check if emulation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, EmulationResult::Success { .. })
    }

    /// Get execution time
    pub fn execution_time(&self) -> std::time::Duration {
        match self {
            EmulationResult::Success { execution_time } => *execution_time,
            EmulationResult::Error { execution_time, .. } => *execution_time,
        }
    }

    /// Get error message if failed
    pub fn error(&self) -> Option<&str> {
        match self {
            EmulationResult::Success { .. } => None,
            EmulationResult::Error { error, .. } => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_config_default() {
        let config = EmulatorConfig::default();
        assert!(!config.debug);
        assert_eq!(config.max_instructions, 1000000);
        assert_eq!(config.timeout, 1000000);
    }

    #[test]
    fn test_emulation_result() {
        let success = EmulationResult::Success {
            execution_time: std::time::Duration::from_millis(100),
        };
        assert!(success.is_success());
        assert!(success.error().is_none());

        let error = EmulationResult::Error {
            error: "Test error".to_string(),
            execution_time: std::time::Duration::from_millis(50),
        };
        assert!(!error.is_success());
        assert_eq!(error.error(), Some("Test error"));
    }
}
