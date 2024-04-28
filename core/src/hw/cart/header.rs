//! Game ROM cartridge header.

use std::array::TryFromSliceError;
use std::fmt::Display;
use std::str::Utf8Error;

use log::warn;
use thiserror::Error;

/// Nintendo logo.
///
/// ```text
/// ██▄  ██ ██        ▄▄                   ██
/// ██▀▄ ██ ▄▄ ▄▄ ▄▄ ▀██▀ ▄▄▄▄  ▄▄ ▄▄   ▄▄▄██  ▄▄▄▄
/// ██ ▀▄██ ██ ██▀ ██ ██ ██▄▄██ ██▀ ██ ██  ██ ██  ██
/// ██  ▀██ ██ ██  ██ ██ ▀█▄▄▄▄ ██  ██ ▀█▄▄██ ▀█▄▄█▀
/// ```
///
/// Compressed copy of Nintendo's logo rendered by the boot ROM. The console
/// will refuse to pass control to cartridges that do not contain an exact copy
/// of this data.
pub const LOGO: [u8; 0x30] = [
    0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d,
    0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99,
    0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
];

/// Cartridge header.
///
/// Information about the ROM and the cartridge containing it. Stored in the
/// byte range `[0x100, 0x150)`.
#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Header {
    /// Equality with boot ROM's Nintendo logo.
    pub logo: bool,
    /// Title of this ROM.
    pub title: Option<String>,
    /// DMG model support.
    pub dmg: bool,
    /// CGB model support.
    pub cgb: bool,
    /// SGB model support.
    pub sgb: bool,
    /// Cartridge hardware.
    pub cart: Kind,
    /// ROM size in bytes.
    pub romsz: usize,
    /// ROM size in bytes.
    pub ramsz: usize,
    /// Destination code (Japan/Worldwide)
    pub jpn: bool,
    /// Revision number of this ROM.
    pub version: u8,
    /// 8-bit header checksum.
    pub hchk: u8,
    /// 16-bit global checksum.
    pub gchk: u16,
}

impl Header {
    /// Constructs a new `Header`.
    ///
    /// For detailed information on how Game Boy headers are parsed, see [Pan
    /// Docs][pandocs].
    ///
    /// # Errors
    ///
    /// Returns an error if the header could not be parsed from the ROM.
    ///
    /// [pandocs]: https://gbdev.io/pandocs/The_Cartridge_Header.html
    pub fn new(rom: &[u8]) -> Result<Self> {
        // Extract header bytes
        let head: &[u8; 0x50] = rom
            .get(0x100..0x150)
            .ok_or(Error::Missing)?
            .try_into()
            .map_err(Error::Slice)?;

        // Compare logo data
        let logo = head[0x04..=0x33] == LOGO;
        // Parse title
        let tlen = if head[0x43] & 0x80 == 0 { 16 } else { 15 };
        let title = match std::str::from_utf8(&head[0x34..0x34 + tlen])
            .map_err(Error::Title)?
            .trim_matches('\0')
        {
            "" => None,
            ok => Some(ok),
        }
        .map(ToString::to_string);
        // Parse CGB flag
        let dmg = (head[0x43] & 0xc0) != 0xc0;
        let cgb = match head[0x43] & 0xbf {
            0x00 => Ok(false),
            0x80 => Ok(true),
            byte => Err(Error::Color(byte)),
        }?;
        // Parse SGB flag
        let sgb = match head[0x46] {
            0x00 => false,
            0x03 => true,
            byte => {
                warn!("non-standard SGB flag: {byte:#04x}");
                false
            }
        };
        // Parse cartridge kind
        let cart = head[0x47].try_into()?;
        // Parse ROM size
        let romsz = match head[0x48] {
            byte @ 0x00..=0x08 => Ok(0x8000 << byte),
            byte => Err(Error::Rom(byte)),
        }?;
        // Parse RAM size
        let ramsz = match head[0x49] {
            0x00 => Ok(0),
            0x01 => Ok(0x800),
            0x02 => Ok(0x2000),
            0x03 => Ok(0x8000),
            0x04 => Ok(0x20000),
            0x05 => Ok(0x10000),
            byte => Err(Error::Ram(byte)),
        }?;
        // Parse RAM size
        let jpn = match head[0x4a] {
            0x00 => Ok(true),
            0x01 => Ok(false),
            byte => Err(Error::Region(byte)),
        }?;
        // Parse mark ROM version number
        let version = head[0x4c];
        // Parse header checksum
        let hchk = head[0x4d];
        // Parse global checksum
        let gchk = u16::from_be_bytes([head[0x4e], head[0x4f]]);

        // Verify header checksum
        let chk = Self::hchk(rom);
        if chk != hchk {
            return Err(Error::HeaderChk {
                found: chk,
                expected: hchk,
            });
        }
        // Verify global checksum
        let chk = Self::gchk(rom);
        if chk != gchk {
            warn!("global checksum failed: {chk:#06x} != {gchk:#06x}");
        }

        Ok(Self {
            logo,
            title,
            dmg,
            cgb,
            sgb,
            cart,
            romsz,
            ramsz,
            jpn,
            version,
            hchk,
            gchk,
        })
    }

    /// Constructs a blank `Header`.
    #[must_use]
    pub(super) fn blank() -> Self {
        Self {
            logo: false,
            title: None,
            dmg: false,
            cgb: false,
            sgb: false,
            cart: Kind::None {
                ram: false,
                pwr: false,
            },
            romsz: 0x8000,
            ramsz: 0x00,
            jpn: false,
            version: 0x00,
            hchk: 0x00,
            gchk: 0x00,
        }
    }

    /// Checks header integrity.
    ///
    /// # Errors
    ///
    /// Returns an error when the cartridge header is invalid.
    pub(super) fn check(rom: &[u8]) -> Result<()> {
        // Extract the header bytes
        let head: &[u8; 0x50] = rom
            .get(0x100..0x150)
            .ok_or(Error::Missing)?
            .try_into()
            .map_err(Error::Slice)?;

        // Verify header checksum
        let hchk = head[0x4d];
        let chk = Self::hchk(rom);
        if hchk != chk {
            return Err(Error::HeaderChk {
                found: chk,
                expected: hchk,
            });
        }

        // Verify global checksum
        let gchk = u16::from_be_bytes(head[0x4e..=0x4f].try_into().map_err(Error::Slice)?);
        let chk = Self::gchk(rom);
        if gchk != chk {
            return Err(Error::GlobalChk {
                found: chk,
                expected: gchk,
            });
        }

        // Everything looks good
        Ok(())
    }

    /// Calculates header checksum.
    fn hchk(rom: &[u8]) -> u8 {
        rom[0x134..=0x14c]
            .iter()
            .copied()
            .fold(0u8, |accum, item| accum.wrapping_sub(item).wrapping_sub(1))
    }

    /// Calculates global checksum.
    fn gchk(rom: &[u8]) -> u16 {
        rom.iter()
            .copied()
            .fold(0u16, |accum, item| accum.wrapping_add(item as u16))
            .wrapping_sub(rom[0x14e] as u16)
            .wrapping_sub(rom[0x14f] as u16)
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────────────────┐")?;
        writeln!(f, "│ {:^16} │", self.title.as_deref().unwrap_or("Unknown"))?;
        writeln!(f, "├──────────────────┤")?;
        writeln!(
            f,
            "│ Model: {:>9} │",
            match (self.dmg, self.cgb) {
                (false, false) => "None",
                (false, true) => "CGB",
                (true, false) => "DMG",
                (true, true) => "DMG + CGB",
            }
        )?;
        writeln!(f, "│ SGB: {:>11} │", self.sgb)?;
        writeln!(f, "├──────────────────┤")?;
        writeln!(f, "│ MBC: {:>11} │", self.cart)?;
        writeln!(f, "├──────────────────┤")?;
        writeln!(f, "│ ROM: {:>9} B │", self.romsz)?;
        writeln!(f, "│ RAM: {:>9} B │", self.ramsz)?;
        writeln!(f, "├──────────────────┤")?;
        writeln!(
            f,
            "│ Region: {:>8} │",
            if self.jpn { "Japan" } else { "World" }
        )?;
        writeln!(
            f,
            "│ Version: {:>7} │",
            format!(
                "v{}.{}",
                ((self.version & 0xf0) >> 4) + 1,
                self.version & 0x0f
            )
        )?;
        writeln!(f, "├──────────────────┤")?;
        writeln!(f, "│ Header:       {:0>2x} │", self.hchk)?;
        writeln!(f, "│ Global:     {:0>4x} │", self.gchk)?;
        write!(f, "└──────────────────┘")
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(rom: &[u8]) -> Result<Self, Self::Error> {
        Self::new(rom)
    }
}

/// Cartridge information.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kind {
    None { ram: bool, pwr: bool },
    Mbc1 { ram: bool, pwr: bool },
    Mbc2 { pwr: bool },
    Mbc3 { ram: bool, pwr: bool, rtc: bool },
    Mbc5 { ram: bool, pwr: bool, vib: bool },
    Mbc6,
    Mbc7,
    Mmm01 { ram: bool, pwr: bool },
    M161,
    HuC1,
    HuC3,
    Camera,
}

impl Display for Kind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None   { .. } => "None",
            Self::Mbc1   { .. } => "MBC1",
            Self::Mbc2   { .. } => "MBC2",
            Self::Mbc3   { .. } => "MBC3",
            Self::Mbc5   { .. } => "MBC5",
            Self::Mbc6   { .. } => "MBC6",
            Self::Mbc7   { .. } => "MBC7",
            Self::Mmm01  { .. } => "MMM01",
            Self::M161   { .. } => "M161",
            Self::HuC1   { .. } => "HuC1",
            Self::HuC3   { .. } => "HuC3",
            Self::Camera { .. } => "Camera",
        }
        .fmt(f)
    }
}

impl TryFrom<u8> for Kind {
    type Error = Error;

    #[allow(clippy::too_many_lines)]
    #[rustfmt::skip]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Kind::None {
                ram: false,
                pwr: false,
            }),
            0x01 => Ok(Kind::Mbc1 {
                ram: false,
                pwr: false,
            }),
            0x02 => Ok(Kind::Mbc1 {
                ram: true,
                pwr: false,
            }),
            0x03 => Ok(Kind::Mbc1 {
                ram: true,
                pwr: true,
            }),
            0x05 => Ok(Kind::Mbc2 {
                pwr: false
            }),
            0x06 => Ok(Kind::Mbc2 {
                pwr: true
            }),
            0x08 => Ok(Kind::None {
                ram: true,
                pwr: false,
            }),
            0x09 => Ok(Kind::None {
                ram: true,
                pwr: true,
            }),
            0x0b => Ok(Kind::Mmm01 {
                ram: false,
                pwr: false,
            }),
            0x0c => Ok(Kind::Mmm01 {
                ram: true,
                pwr: false,
            }),
            0x0d => Ok(Kind::Mmm01 {
                ram: true,
                pwr: true,
            }),
            0x0f => Ok(Kind::Mbc3 {
                ram: false,
                pwr: true,
                rtc: true,
            }),
            0x10 => Ok(Kind::Mbc3 {
                ram: true,
                pwr: true,
                rtc: true,
            }),
            0x11 => Ok(Kind::Mbc3 {
                ram: false,
                pwr: false,
                rtc: false,
            }),
            0x12 => Ok(Kind::Mbc3 {
                ram: true,
                pwr: false,
                rtc: false,
            }),
            0x13 => Ok(Kind::Mbc3 {
                ram: true,
                pwr: true,
                rtc: false,
            }),
            0x19 => Ok(Kind::Mbc5 {
                ram: false,
                pwr: false,
                vib: false,
            }),
            0x1a => Ok(Kind::Mbc5 {
                ram: true,
                pwr: false,
                vib: false,
            }),
            0x1b => Ok(Kind::Mbc5 {
                ram: true,
                pwr: true,
                vib: false,
            }),
            0x1c => Ok(Kind::Mbc5 {
                ram: false,
                pwr: false,
                vib: true,
            }),
            0x1d => Ok(Kind::Mbc5 {
                ram: true,
                pwr: false,
                vib: true,
            }),
            0x1e => Ok(Kind::Mbc5 {
                ram: true,
                pwr: true,
                vib: true,
            }),
            0x20 => Ok(Kind::Mbc6),
            0x22 => Ok(Kind::Mbc7),
            0xfc => Ok(Kind::Camera),
            0xfe => Ok(Kind::HuC3),
            0xff => Ok(Kind::HuC1),
            value => Err(Error::Kind(value)),
        }
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A type specifying categories of [`Header`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("missing header bytes")]
    Missing,
    #[error(transparent)]
    Slice(#[from] TryFromSliceError),
    #[error("invalid bytes in title")]
    Title(#[from] Utf8Error),
    #[error("invalid CGB flag: {0:#04x}")]
    Color(u8),
    #[error("unknown hardware: {0:#04x}")]
    Kind(u8),
    #[error("invalid ROM size: {0:#04x}")]
    Rom(u8),
    #[error("invalid RAM size: {0:#04x}")]
    Ram(u8),
    #[error("invalid region: {0:#04x}")]
    Region(u8),
    #[error("bad header checksum (found {found:#04x}, expected {expected:#04x})")]
    HeaderChk { found: u8, expected: u8 },
    #[error("bad global checksum (found {found:#06x}, expected {expected:#06x})")]
    GlobalChk { found: u16, expected: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample ROM header.
    const HEAD: &[u8; 0x0150] = include_bytes!("../../../../roms/header/basic.gb");

    #[test]
    fn parse_works() {
        // Parse header
        let parse = Header::try_from(&HEAD[..]).unwrap();
        // Hard-code expected
        let truth = Header {
            logo: true,
            dmg: true,
            cart: Kind::None {
                ram: true,
                pwr: false,
            },
            romsz: 0x8000,
            ramsz: 0x2000,
            hchk: 0xdc,
            gchk: 0x31bb,
            ..Header::blank()
        };

        assert_eq!(parse, truth);
    }
}
