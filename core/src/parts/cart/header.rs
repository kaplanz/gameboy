//! Game ROM cartridge header.
//!
//! Encoded in the ROM at the address range `[$0100..$0150]` is the header, which
//! encodes both physical attributes describing the hardware of the cartridge,
//! flags describing console support, and characteristics of the software.

use std::array::TryFromSliceError;
use std::fmt::Display;
use std::str::Utf8Error;

use log::warn;
use remus::{Byte, Word};
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
pub const LOGO: [Byte; 0x30] = [
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
    /// Hardware information.
    pub info: Info,
    /// ROM size in bytes.
    pub romsz: usize,
    /// ROM size in bytes.
    pub ramsz: usize,
    /// Destination code (Japan/Worldwide)
    pub jpn: bool,
    /// Revision number of this ROM.
    pub version: Byte,
    /// 8-bit header checksum.
    pub hchk: Byte,
    /// 16-bit global checksum.
    pub gchk: Word,
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
    pub fn new(rom: &[Byte]) -> Result<Self> {
        // Extract header bytes
        let head: &[Byte; 0x50] = rom
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
        let gchk = Word::from_be_bytes([head[0x4e], head[0x4f]]);

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
            info: cart,
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
            info: Info::Bare {
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
    pub(super) fn check(rom: &[Byte]) -> Result<()> {
        // Extract the header bytes
        let head: &[Byte; 0x50] = rom
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
        let gchk = Word::from_be_bytes(head[0x4e..=0x4f].try_into().map_err(Error::Slice)?);
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
    fn hchk(rom: &[Byte]) -> Byte {
        rom[0x134..=0x14c]
            .iter()
            .copied()
            .fold(0, |accum, item| accum.wrapping_sub(item).wrapping_sub(1))
    }

    /// Calculates global checksum.
    fn gchk(rom: &[Byte]) -> Word {
        rom.iter()
            .copied()
            .fold(0u16, |accum, item| accum.wrapping_add(Word::from(item)))
            .wrapping_sub(Word::from(rom[0x14e]))
            .wrapping_sub(Word::from(rom[0x14f]))
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
        writeln!(f, "│ MBC: {:>11} │", self.info)?;
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

impl TryFrom<&[Byte]> for Header {
    type Error = Error;

    fn try_from(rom: &[Byte]) -> Result<Self, Self::Error> {
        Self::new(rom)
    }
}

/// Hardware information.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Info {
    Bare { ram: bool, pwr: bool },
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

impl Info {
    /// Check if the cartridge has a battery.
    #[must_use]
    pub fn has_battery(&self) -> bool {
        match self {
            Info::Bare { pwr, .. }
            | Info::Mbc1 { pwr, .. }
            | Info::Mbc2 { pwr, .. }
            | Info::Mbc3 { pwr, .. }
            | Info::Mbc5 { pwr, .. }
            | Info::Mmm01 { pwr, .. } => *pwr,
            _ => false,
        }
    }

    /// Check if the cartridge has any RAM.
    #[must_use]
    pub fn has_ram(&self) -> bool {
        match self {
            Info::Bare { ram, .. }
            | Info::Mbc1 { ram, .. }
            | Info::Mbc3 { ram, .. }
            | Info::Mbc5 { ram, .. }
            | Info::Mmm01 { ram, .. } => *ram,
            _ => false,
        }
    }
}

impl Display for Info {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bare   { .. } => "None",
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

impl TryFrom<Byte> for Info {
    type Error = Error;

    #[allow(clippy::too_many_lines)]
    #[rustfmt::skip]
    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Info::Bare {
                ram: false,
                pwr: false,
            }),
            0x01 => Ok(Info::Mbc1 {
                ram: false,
                pwr: false,
            }),
            0x02 => Ok(Info::Mbc1 {
                ram: true,
                pwr: false,
            }),
            0x03 => Ok(Info::Mbc1 {
                ram: true,
                pwr: true,
            }),
            0x05 => Ok(Info::Mbc2 {
                pwr: false
            }),
            0x06 => Ok(Info::Mbc2 {
                pwr: true
            }),
            0x08 => Ok(Info::Bare {
                ram: true,
                pwr: false,
            }),
            0x09 => Ok(Info::Bare {
                ram: true,
                pwr: true,
            }),
            0x0b => Ok(Info::Mmm01 {
                ram: false,
                pwr: false,
            }),
            0x0c => Ok(Info::Mmm01 {
                ram: true,
                pwr: false,
            }),
            0x0d => Ok(Info::Mmm01 {
                ram: true,
                pwr: true,
            }),
            0x0f => Ok(Info::Mbc3 {
                ram: false,
                pwr: true,
                rtc: true,
            }),
            0x10 => Ok(Info::Mbc3 {
                ram: true,
                pwr: true,
                rtc: true,
            }),
            0x11 => Ok(Info::Mbc3 {
                ram: false,
                pwr: false,
                rtc: false,
            }),
            0x12 => Ok(Info::Mbc3 {
                ram: true,
                pwr: false,
                rtc: false,
            }),
            0x13 => Ok(Info::Mbc3 {
                ram: true,
                pwr: true,
                rtc: false,
            }),
            0x19 => Ok(Info::Mbc5 {
                ram: false,
                pwr: false,
                vib: false,
            }),
            0x1a => Ok(Info::Mbc5 {
                ram: true,
                pwr: false,
                vib: false,
            }),
            0x1b => Ok(Info::Mbc5 {
                ram: true,
                pwr: true,
                vib: false,
            }),
            0x1c => Ok(Info::Mbc5 {
                ram: false,
                pwr: false,
                vib: true,
            }),
            0x1d => Ok(Info::Mbc5 {
                ram: true,
                pwr: false,
                vib: true,
            }),
            0x1e => Ok(Info::Mbc5 {
                ram: true,
                pwr: true,
                vib: true,
            }),
            0x20 => Ok(Info::Mbc6),
            0x22 => Ok(Info::Mbc7),
            0xfc => Ok(Info::Camera),
            0xfe => Ok(Info::HuC3),
            0xff => Ok(Info::HuC1),
            value => Err(Error::Kind(value)),
        }
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error caused by parsing a cartridge [header](Header).
#[derive(Debug, Error)]
pub enum Error {
    /// Missing header data.
    #[error("missing header data")]
    Missing,
    /// SLice to array conversion failed.
    #[error(transparent)]
    Slice(#[from] TryFromSliceError),
    /// Invalid bytes in title.
    #[error("invalid bytes in title")]
    Title(#[from] Utf8Error),
    /// Invalid CGB flag.
    #[error("invalid CGB flag: {0:#04x}")]
    Color(Byte),
    /// Unknown hardware.
    #[error("unknown hardware: {0:#04x}")]
    Kind(Byte),
    /// Invalid ROM size.
    #[error("invalid ROM size: {0:#04x}")]
    Rom(Byte),
    /// Invalid RAM size.
    #[error("invalid RAM size: {0:#04x}")]
    Ram(Byte),
    /// Invalid region.
    #[error("invalid region: {0:#04x}")]
    Region(Byte),
    /// Bad header checksum.
    #[error("bad header checksum (found {found:#04x}, expected {expected:#04x})")]
    HeaderChk { found: Byte, expected: Byte },
    /// Bad global checksum.
    #[error("bad global checksum (found {found:#06x}, expected {expected:#06x})")]
    GlobalChk { found: Word, expected: Word },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample ROM header.
    const HEAD: &[Byte; 0x0150] = include_bytes!("../../../../roms/header/basic.gb");

    #[test]
    fn parse_works() {
        // Parse header
        let parse = Header::try_from(&HEAD[..]).unwrap();
        // Hard-code expected
        let truth = Header {
            logo: true,
            dmg: true,
            info: Info::Bare {
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
