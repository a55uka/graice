use crate::error::{GraiceError, Result};
use crate::header::{N64RomHeader, parse_z64_header};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// ROM format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomFormat {
    /// .z64 - Big Endian (native N64 format)
    Z64,
    /// .v64 - Byte Swapped
    V64,
    /// .n64 - Little Endian
    N64,
}

impl RomFormat {
    /// Detect ROM format from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "z64" => Some(Self::Z64),
            "v64" => Some(Self::V64),
            "n64" => Some(Self::N64),
            _ => None,
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Z64 => "z64",
            Self::V64 => "v64",
            Self::N64 => "n64",
        }
    }
}

/// N64 ROM structure containing header and data
#[derive(Debug)]
pub struct N64Rom<'a> {
    pub header: N64RomHeader<'a>,
    pub data: Vec<u8>,
    pub format: RomFormat,
}

impl<'a> N64Rom<'a> {
    /// Load a ROM from file
    pub fn load_from_file(path: &Path) -> Result<N64Rom<'static>> {
        let format = RomFormat::from_extension(path)
            .ok_or_else(|| GraiceError::ParseError("Unknown ROM format".to_string()))?;

        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Self::from_bytes(data, format)
    }

    /// Create ROM from byte data
    pub fn from_bytes(mut data: Vec<u8>, format: RomFormat) -> Result<N64Rom<'static>> {
        match format {
            RomFormat::Z64 => {
                // Correct format (Big Endian)
            }
            RomFormat::V64 => {
                // Byte swap every 2 bytes (Swapped Endian)
                Self::byte_swap_v64(&mut data);
            }
            RomFormat::N64 => {
                // Convert from little endian to big endian (Little Endian)
                Self::convert_n64_to_z64(&mut data);
            }
        }

        let header = parse_z64_header(&data)?;

        // We need to use unsafe here to extend lifetime
        let header_static = unsafe { std::mem::transmute(header) };

        Ok(N64Rom {
            header: header_static,
            data,
            format,
        })
    }

    /// Get ROM data without header
    pub fn rom_data(&self) -> &[u8] {
        &self.data[N64RomHeader::HEADER_SIZE..]
    }

    /// Get the full ROM data including header
    pub fn full_data(&self) -> &[u8] {
        &self.data
    }

    /// Get ROM size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if ROM is standard
    pub fn is_standard(&self) -> bool {
        self.header.is_standard_magic()
    }

    /// Get boot address for emulation
    pub fn boot_address(&self) -> u64 {
        self.header.boot_address()
    }

    /// Byte swap for V64 format conversion
    fn byte_swap_v64(data: &mut [u8]) {
        for chunk in data.chunks_exact_mut(2) {
            chunk.swap(0, 1);
        }
    }

    /// Convert N64 format to Z64 format
    fn convert_n64_to_z64(data: &mut [u8]) {
        // N64 format is little endian, need to convert to big endian
        for chunk in data.chunks_exact_mut(4) {
            chunk.reverse();
        }
    }

    /// Print ROM information
    pub fn print_info(&self) {
        println!("ROM Information:");
        println!("Format: {:?}", self.format);
        println!(
            "Size: {} bytes ({:.2} MB)",
            self.size(),
            self.size() as f64 / 0x100_000 as f64
        );
        println!("Game Title: '{}'", self.header.game_title_trimmed());
        println!("Boot Point: 0x{:08x}", self.header.boot_point);
        println!("Clock Rate: {} Hz", self.header.clock_rate);
        println!("ROM Version: {}", self.header.rom_version);
        println!("LibUltra Version: 0x{:08x}", self.header.libultra_version);
        println!("CRC: {:02x?}", self.header.crc);

        let game_code = &self.header.game_code;
        println!(
            "Game Code: {} ({:?}) - {} - {} ({:?})",
            game_code.category,
            game_code.category_code(),
            game_code.unique,
            game_code.country,
            game_code.destination_region()
        );

        if !self.header.is_standard_magic() {
            println!("WARNING: ROM has non-standard magic values");
        }
    }
}

/// ROM loader utility
pub struct RomLoader;

impl RomLoader {
    /// Load ROM from path with automatic format detection
    pub fn load<P: AsRef<Path>>(path: P) -> Result<N64Rom<'static>> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(GraiceError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ROM file not found: {}", path.display()),
            )));
        }

        N64Rom::load_from_file(path)
    }

    /// Validate ROM file before loading
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(GraiceError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "ROM file not found",
            )));
        }

        let format = RomFormat::from_extension(path)
            .ok_or_else(|| GraiceError::ParseError("Unknown ROM format".to_string()))?;

        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();

        if size < N64RomHeader::HEADER_SIZE as u64 {
            return Err(GraiceError::InvalidHeader(
                "ROM file too small to contain header".to_string(),
            ));
        }

        println!("ROM validation passed:");
        println!("  Path: {}", path.display());
        println!("  Format: {:?}", format);
        println!("  Size: {} bytes", size);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_format_detection() {
        let path = Path::new("test.z64");
        assert_eq!(RomFormat::from_extension(path), Some(RomFormat::Z64));

        let path = Path::new("test.v64");
        assert_eq!(RomFormat::from_extension(path), Some(RomFormat::V64));

        let path = Path::new("test.n64");
        assert_eq!(RomFormat::from_extension(path), Some(RomFormat::N64));

        let path = Path::new("test.txt");
        assert_eq!(RomFormat::from_extension(path), None);
    }

    #[test]
    fn test_byte_swap_v64() {
        let mut data = vec![0x12, 0x34, 0x56, 0x78];
        N64Rom::byte_swap_v64(&mut data);
        assert_eq!(data, vec![0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    fn test_convert_n64_to_z64() {
        let mut data = vec![0x12, 0x34, 0x56, 0x78];
        N64Rom::convert_n64_to_z64(&mut data);
        assert_eq!(data, vec![0x78, 0x56, 0x34, 0x12]);
    }
}
