# Graice - Nintendo 64 and Emulator

A Rust-based Nintendo 64 emulator using the Unicorn Engine.

## Features

- **ROM Format Support**: Supports .z64 (big-endian), .v64 (byte-swapped), and .n64 (little-endian) formats
- **Header Parsing**: Complete N64 ROM header parsing with validation
- **Emulation**: MIPS64 CPU emulation using Unicorn Engine
- **Memory Mapping**: Accurate N64 memory map implementation
- **Debugging**: Built-in debugging features and register inspection

## Project Structure

```
src/
├── main.rs           # Main application entry point
├── lib.rs            # Library exports and module declarations
├── error.rs          # Custom error types and handling
├── header.rs         # N64 ROM header structures and parsing
├── rom.rs            # ROM loading and format conversion
└── emulator.rs       # N64 emulation logic using Unicorn Engine
```

## Dependencies

- `scroll` - Binary parsing library
- `unicorn-engine` - CPU emulation engine

## Usage

```bash
cargo run <rom_file>
```

Example:
```bash
cargo run "roms/Mario Kart 64 (Europe) (Rev 1).z64"
```

## ROM Formats

The emulator supports three common N64 ROM formats:

- **Z64**: Big-endian format (native N64 byte order)
- **V64**: Byte-swapped format (16-bit words swapped)
- **N64**: Little-endian format (32-bit words in little-endian order)

All formats are converted to Z64 format internally.

## Architecture

### Header Module (`header.rs`)
- `N64RomHeader`: Main ROM header structure
- `PiBsDom1Config`: PI BSD Domain 1 configuration
- `GameCode`: Game identification structure
- `CategoryCode`: Game category enumeration
- `DestinationRegion`: Region code enumeration

### ROM Module (`rom.rs`)
- `N64Rom`: ROM container with header and data
- `RomLoader`: Utility for loading ROMs from files
- `RomFormat`: Format detection and conversion

### Emulator Module (`emulator.rs`)
- `N64Emulator`: Main emulator class
- `EmulatorConfig`: Configuration options
- `EmulationResult`: Execution results

### Error Module (`error.rs`)
- `GraiceError`: Custom error types
- Error handling

## Memory Map

The emulator implements the standard N64 memory map:

- **RDRAM**: `0x00000000` - `0x007FFFFF` (8MB, read/write)
- **ROM**: `0x10000000` - `0x13FFFFFF` (64MB max, read/execute)
- **SP DMEM**: `0x04000000` - `0x04000FFF` (4KB, read/write)
- **SP IMEM**: `0x04001000` - `0x04001FFF` (4KB, read/write/execute)

## Configuration

The emulator can be configured with:

```rust
EmulatorConfig {
    debug: true,              // Enable debug output
    max_instructions: 100000, // Maximum instructions to execute
    timeout: 5000000,         // Timeout in microseconds
}
```

## Example Output

```
ROM validation passed:
  Path: .\roms\Mario Kart 64 (Europe) (Rev 1).z64
  Format: Z64
  Size: 12582912 bytes
ROM Information:
Format: Z64
Size: 12582912 bytes (12.00 MB)
Game Title: 'MARIOKART64'
Boot Point: 0x80000400
Clock Rate: 15 Hz
ROM Version: 1
LibUltra Version: 0x00001446
CRC: [25, 77, c7, d4, d1, 8f, aa, ae]
Game Code: N (Some(GamePak)) - KT - P (Some(Europe))
Memory maps initialized:
  RDRAM: 0x00000000 - 0x00800000
  ROM:   0x10000000 - 0x14000000
  SP DMEM: 0x04000000 - 0x04001000
  SP IMEM: 0x04001000 - 0x04002000
ROM loaded: 12582912 bytes at 0x10000000
Boot point: 0x80000400

Starting emulation...
Starting emulation:
  Boot address: 0x10000400
  End address: 0x10bff000
Emulation completed successfully in 6.6518ms
Emulation finished in 6.6518ms
Register states:
  $t0: 0xffffffffa40004f8 (18446744072166049016)
  $t1: 0xffffffffa0000034 (18446744072098938932)
  $t2: 0xffffffffa4000000 (18446744072166047744)
  $t3: 0xffffffffa4000774 (18446744072166049652)
  $t4: 0x0000000000000000 (0)
  $t5: 0x0000000000000000 (0)
  $t6: 0x0000000000000000 (0)
  $t7: 0x0000000000000000 (0)
  $t8: 0x0000000000000000 (0)
  $t9: 0x0000000000000000 (0)
  $pc: 0x00000000100004a0 (268436640)
  $sp: 0x0000000000000000 (0)
  $ra: 0x0000000000000000 (0)
```

## Testing

Run tests with:
```bash
cargo test
```
