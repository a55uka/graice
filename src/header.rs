use crate::error::Result;
use scroll::ctx::{StrCtx, TryFromCtx};
use scroll::{Endian, Pread};

/// PI BSD Domain 1 Configuration
#[derive(Debug, Clone)]
pub struct PiBsDom1Config {
    pub release_timing: u8, // Release Timing (bits 4-5)
    pub page_size: u8,     // Page Size (bits 0-3)
    pub pulse_width: u8,  // Pulse Width
    pub latency: u8,     // Latency
}

impl PiBsDom1Config {
    /// Standard N64 magic values
    pub const STANDARD_RELEASE_TIMING: u8 = 0x3;
    pub const STANDARD_PAGE_SIZE: u8 = 0x7;
    pub const STANDARD_PULSE_WIDTH: u8 = 0x12;
    pub const STANDARD_LATENCY: u8 = 0x40;

    /// Check if the configuration matches standard N64 values
    pub fn is_standard(&self) -> bool {
        self.release_timing == Self::STANDARD_RELEASE_TIMING
            && self.page_size == Self::STANDARD_PAGE_SIZE
            && self.pulse_width == Self::STANDARD_PULSE_WIDTH
            && self.latency == Self::STANDARD_LATENCY
    }
}

impl<'a> TryFromCtx<'a, Endian> for PiBsDom1Config {
    type Error = scroll::Error;

    fn try_from_ctx(
        from: &'a [u8],
        ctx: Endian,
    ) -> std::result::Result<(Self, usize), Self::Error> {
        if from.len() < 3 {
            return Err(scroll::Error::Custom(
                "Insufficient bytes for PiBsDom1Config".to_string(),
            ));
        }

        let offset = &mut 0;
        let pi_config: u8 = from.gread_with(offset, ctx)?;
        let pulse_width = from.gread_with(offset, ctx)?;
        let latency = from.gread_with(offset, ctx)?;

        let release_timing = (pi_config >> 4) & 0x3; // Bits 4-5
        let page_size = pi_config & 0xF; // Bits 0-3

        Ok((
            Self {
                release_timing,
                page_size,
                pulse_width,
                latency,
            },
            *offset,
        ))
    }
}

/// Game code structure containing category, unique ID, and country
#[derive(Debug, Clone)]
pub struct GameCode<'a> {
    pub category: char,
    pub unique: &'a str,
    pub country: char,
}

impl<'a> GameCode<'a> {
    /// Get the category code enum
    pub fn category_code(&self) -> Option<CategoryCode> {
        CategoryCode::from_char(self.category)
    }

    /// Get the destination region enum
    pub fn destination_region(&self) -> Option<DestinationRegion> {
        DestinationRegion::from_char(self.country)
    }
}

impl<'a> TryFromCtx<'a, Endian> for GameCode<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(
        from: &'a [u8],
        ctx: Endian,
    ) -> std::result::Result<(Self, usize), Self::Error> {
        if from.len() < 4 {
            return Err(scroll::Error::Custom(
                "Insufficient bytes for GameCode".to_string(),
            ));
        }

        let offset = &mut 0;
        let category = from.gread_with::<u8>(offset, ctx)? as char;
        let unique: &str = from.gread_with(offset, StrCtx::Length(2))?;
        let country = from.gread_with::<u8>(offset, ctx)? as char;

        Ok((
            Self {
                category,
                unique,
                country,
            },
            *offset,
        ))
    }
}

/// Nintendo 64 ROM header structure
#[derive(Debug, Clone)]
pub struct N64RomHeader<'a> {
    pub reserved1: u8, // Usually 0x80
    pub pi_bsd_dom1_config: PiBsDom1Config,
    pub clock_rate: u32,
    pub boot_point: u32,
    pub libultra_version: u32,
    pub crc: [u8; 8],
    pub reserved2: [u8; 8],
    pub game_title: &'a str,
    pub reserved3: [u8; 7],
    pub game_code: GameCode<'a>,
    pub rom_version: u8,
    pub ipl3_code: [u8; 0xFC0],
}

impl<'a> N64RomHeader<'a> {
    /// Standard first reserved byte value
    pub const STANDARD_RESERVED1: u8 = 0x80;

    /// Header size in bytes
    pub const HEADER_SIZE: usize = 0x1000;

    /// Check if the header has standard magic values
    pub fn is_standard_magic(&self) -> bool {
        self.reserved1 == Self::STANDARD_RESERVED1 && self.pi_bsd_dom1_config.is_standard()
    }

    /// Get the game title with whitespace trimmed
    pub fn game_title_trimmed(&self) -> &str {
        self.game_title.trim()
    }

    /// Get the boot address adjusted for emulation
    pub fn boot_address(&self) -> u64 {
        self.boot_point as u64
    }
}

impl<'a> TryFromCtx<'a, Endian> for N64RomHeader<'a> {
    type Error = scroll::Error;

    fn try_from_ctx(
        from: &'a [u8],
        ctx: Endian,
    ) -> std::result::Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let reserved1 = from.gread_with(offset, ctx)?;
        let pi_bsd_dom1_config = from.gread_with(offset, ctx)?;
        let clock_rate = from.gread_with(offset, ctx)?;
        let boot_point = from.gread_with(offset, ctx)?;
        let libultra_version = from.gread_with(offset, ctx)?;
        let crc = from.gread_with(offset, ctx)?;
        let reserved2 = from.gread_with(offset, ctx)?;
        let game_title = from.gread_with(offset, StrCtx::Length(0x14))?;
        let reserved3 = from.gread_with(offset, ctx)?;
        let game_code = from.gread_with(offset, ctx)?;
        let rom_version = from.gread_with(offset, ctx)?;
        let ipl3_code = from.gread_with(offset, ctx)?;

        Ok((
            Self {
                reserved1,
                pi_bsd_dom1_config,
                clock_rate,
                boot_point,
                libultra_version,
                crc,
                reserved2,
                game_title,
                reserved3,
                game_code,
                rom_version,
                ipl3_code,
            },
            *offset,
        ))
    }
}

/// Category code enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryCode {
    GamePak,
    X64DDDisk,
    ExpandGamePak,
    ExpandX64DDDisk,
    Aleck64GamePak,
}

impl CategoryCode {
    const IDENTIFIERS: [(char, Self); 5] = [
        ('N', Self::GamePak),
        ('D', Self::X64DDDisk),
        ('C', Self::ExpandGamePak),
        ('E', Self::ExpandX64DDDisk),
        ('Z', Self::Aleck64GamePak),
    ];

    /// Convert a character to a category code
    pub fn from_char(c: char) -> Option<Self> {
        Self::IDENTIFIERS
            .iter()
            .find(|&&(ch, _)| ch == c.to_ascii_uppercase())
            .map(|(_, code)| *code)
    }

    /// Get the character identifier for this category
    pub fn to_char(&self) -> char {
        Self::IDENTIFIERS
            .iter()
            .find(|&&(_, code)| code == *self)
            .map(|&(ch, _)| ch)
            .unwrap()
    }
}

/// Destination region enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationRegion {
    All,
    Brazil,
    China,
    Germany,
    NorthAmerica,
    France,
    NTSC,
    Netherlands,
    Italy,
    Japan,
    Korea,
    PAL,
    Canada,
    Europe,
    Spain,
    Australia,
    Scandinavia,
}

impl DestinationRegion {
    const IDENTIFIERS: [(char, Self); 20] = [
        ('A', Self::All),
        ('B', Self::Brazil),
        ('C', Self::China),
        ('D', Self::Germany),
        ('E', Self::NorthAmerica),
        ('F', Self::France),
        ('G', Self::NTSC),
        ('H', Self::Netherlands),
        ('I', Self::Italy),
        ('J', Self::Japan),
        ('K', Self::Korea),
        ('L', Self::PAL),
        ('N', Self::Canada),
        ('P', Self::Europe),
        ('S', Self::Spain),
        ('U', Self::Australia),
        ('W', Self::Scandinavia),
        ('X', Self::Europe),
        ('Y', Self::Europe),
        ('Z', Self::Europe),
    ];

    /// Convert a character to a destination region
    pub fn from_char(c: char) -> Option<Self> {
        Self::IDENTIFIERS
            .iter()
            .find(|&&(ch, _)| ch == c.to_ascii_uppercase())
            .map(|(_, region)| *region)
    }

    /// Get the character identifier for this region
    pub fn to_char(&self) -> char {
        Self::IDENTIFIERS
            .iter()
            .find(|&&(_, region)| region == *self)
            .map(|&(ch, _)| ch)
            .unwrap_or('P') // Default 'P' for Europe
    }
}

/// Parse a Z64 (big-endian) ROM header
pub fn parse_z64_header(buffer: &[u8]) -> Result<N64RomHeader<'_>> {
    let header: N64RomHeader = buffer.pread_with(0, scroll::Endian::Big)?;

    if header.reserved1 != N64RomHeader::STANDARD_RESERVED1 {
        eprintln!(
            "WARNING: Uncommon first reserved value: 0x{:02x}",
            header.reserved1
        );
    }

    if !header.pi_bsd_dom1_config.is_standard() {
        eprintln!("WARNING: Uncommon PI BSD Domain 1 configuration");
    }

    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_code_conversion() {
        assert_eq!(CategoryCode::from_char('N'), Some(CategoryCode::GamePak));
        assert_eq!(CategoryCode::from_char('n'), Some(CategoryCode::GamePak));
        assert_eq!(CategoryCode::from_char('X'), None);

        assert_eq!(CategoryCode::GamePak.to_char(), 'N');
    }

    #[test]
    fn test_destination_region_conversion() {
        assert_eq!(
            DestinationRegion::from_char('J'),
            Some(DestinationRegion::Japan)
        );
        assert_eq!(
            DestinationRegion::from_char('j'),
            Some(DestinationRegion::Japan)
        );
        assert_eq!(
            DestinationRegion::from_char('X'),
            Some(DestinationRegion::Europe)
        );

        assert_eq!(DestinationRegion::Japan.to_char(), 'J');
    }

    #[test]
    fn test_pi_config_standard() {
        let standard = PiBsDom1Config {
            release_timing: 0x3,
            page_size: 0x7,
            pulse_width: 0x12,
            latency: 0x40,
        };
        assert!(standard.is_standard());

        let non_standard = PiBsDom1Config {
            release_timing: 0x2,
            page_size: 0x7,
            pulse_width: 0x12,
            latency: 0x40,
        };
        assert!(!non_standard.is_standard());
    }
}
