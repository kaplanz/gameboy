use std::fmt::Display;
use std::str::Utf8Error;

use log::warn;
use thiserror::Error;

/// Cartridge header.
///
/// Information about the ROM and the cartridge containing it. Stored in the
/// byte range `[0x100, 0x150)`.
#[derive(Debug, PartialEq)]
pub struct Header {
    /// Equality with boot ROM's Nintendo logo.
    pub logo: bool,
    /// Title of this ROM.
    pub title: String,
    /// DMG model support.
    pub dmg: bool,
    /// CGB model support.
    pub cgb: bool,
    /// SGB model support.
    pub sgb: bool,
    /// Cartridge type information.
    pub cart: CartridgeType,
    /// Size of this ROM.
    pub romsz: usize,
    /// Size of external RAM.
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

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────────────────┐")?;
        writeln!(
            f,
            "│ {:^16} │",
            match self.title.replace('\0', " ").trim() {
                "" => "Unknown",
                title => title,
            }
        )?;
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
        writeln!(f, "│ Japan: {:>9} │", self.jpn)?;
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
        writeln!(f, "│ Header Chk:   {:0>2x} │", self.hchk)?;
        writeln!(f, "│ Global Chk: {:0>4x} │", self.gchk)?;
        write!(f, "└──────────────────┘")
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = Error;

    fn try_from(rom: &[u8]) -> Result<Self, Self::Error> {
        // Extract the header bytes
        let header: &[u8; 0x50] = rom
            .get(0x100..0x150)
            .ok_or(Error::Missing)?
            .try_into()
            .unwrap();

        // Parse Nintendo logo
        let logo = header[0x04..=0x33]
            == [
                0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c,
                0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6,
                0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc,
                0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
            ];
        // Parse title
        let tlen = if header[0x43] & 0x80 != 0 { 15 } else { 16 };
        let title = std::str::from_utf8(&header[0x34..0x34 + tlen])
            .map_err(Error::Title)?
            .to_string();
        // Parse CGB flag
        let dmg = (header[0x43] & 0xc0) != 0xc0;
        let cgb = match header[0x43] {
            0x00 | 0x40 => Ok(false),
            0x80 | 0xc0 => Ok(true),
            byte => Err(Error::CgbFlag(byte)),
        }?;
        // Parse SGB flag
        let sgb = match header[0x46] {
            0x00 => Ok(false),
            0x03 => Ok(true),
            byte => Err(Error::SgbFlag(byte)),
        }?;
        // Parse cartridge type
        let cart = header[0x47].try_into()?;
        // Parse ROM size
        let romsz = match header[0x48] {
            byte @ 0x00..=0x08 => Ok(0x8000 << byte),
            byte => Err(Error::RomSize(byte)),
        }?;
        // Parse RAM size
        let ramsz = match header[0x49] {
            0x00 => Ok(0x0),
            0x02 => Ok(0x2000),
            0x03 => Ok(0x8000),
            0x04 => Ok(0x20000),
            0x05 => Ok(0x10000),
            byte => Err(Error::RamSize(byte)),
        }?;
        // Parse RAM size
        let jpn = match header[0x4a] {
            0x00 => Ok(true),
            0x01 => Ok(false),
            byte => Err(Error::DestinationCode(byte)),
        }?;
        // Parse mark ROM version number
        let version = header[0x4c];
        // Parse header checksum
        let hchk = header[0x4d];
        // Parse global checksum
        let gchk = u16::from_be_bytes(header[0x4e..=0x4f].try_into().unwrap());

        // Verify header checksum
        match header[0x34..=0x4c]
            .iter()
            .cloned()
            .fold(0u8, |accum, item| accum.wrapping_sub(item).wrapping_sub(1))
        {
            chk if chk == hchk => Ok(()),
            chk => Err(Error::HeaderChecksum {
                found: chk,
                expected: hchk,
            }),
        }?;
        // Verify global checksum
        match rom
            .iter()
            .cloned()
            .fold(0u16, |accum, item| accum.wrapping_add(item as u16))
            .wrapping_sub(header[0x4e] as u16)
            .wrapping_sub(header[0x4f] as u16)
        {
            chk if chk == gchk => (),
            chk => warn!("Global checksum failed: {chk:#06x} != {gchk:#06x}"),
        };

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
}

/// Cartridge information.
#[derive(Debug, PartialEq)]
pub enum CartridgeType {
    NoMbc {
        ram: bool,
        battery: bool,
    },
    Mbc1 {
        ram: bool,
        battery: bool,
    },
    Mbc2 {
        battery: bool,
    },
    Mmm01 {
        ram: bool,
        battery: bool,
    },
    Mbc3 {
        timer: bool,
        ram: bool,
        battery: bool,
    },
    Mbc5 {
        rumble: bool,
        ram: bool,
        battery: bool,
    },
    Mbc6,
    Mbc7 {
        sensor: bool,
        rumble: bool,
        ram: bool,
        battery: bool,
    },
    PocketCamera,
    HuC3,
    HuC1 {
        ram: bool,
        battery: bool,
    },
}

impl Display for CartridgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMbc { .. } => "ROM",
            Self::Mbc1 { .. } => "MBC1",
            Self::Mbc2 { .. } => "MBC2",
            Self::Mmm01 { .. } => "MMM01",
            Self::Mbc3 { .. } => "MBC3",
            Self::Mbc5 { .. } => "MBC5",
            Self::Mbc6 { .. } => "MBC6",
            Self::Mbc7 { .. } => "MBC7",
            Self::PocketCamera { .. } => "Pkt Camera",
            Self::HuC3 { .. } => "HuC3",
            Self::HuC1 { .. } => "HuC1",
        }
        .fmt(f)
    }
}

impl TryFrom<u8> for CartridgeType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(CartridgeType::NoMbc {
                ram: false,
                battery: false,
            }),
            0x01 => Ok(CartridgeType::Mbc1 {
                ram: false,
                battery: false,
            }),
            0x02 => Ok(CartridgeType::Mbc1 {
                ram: true,
                battery: false,
            }),
            0x03 => Ok(CartridgeType::Mbc1 {
                ram: true,
                battery: true,
            }),
            0x05 => Ok(CartridgeType::Mbc2 { battery: false }),
            0x06 => Ok(CartridgeType::Mbc2 { battery: true }),
            0x08 => Ok(CartridgeType::NoMbc {
                ram: true,
                battery: false,
            }),
            0x09 => Ok(CartridgeType::NoMbc {
                ram: true,
                battery: true,
            }),
            0x0b => Ok(CartridgeType::Mmm01 {
                ram: false,
                battery: false,
            }),
            0x0c => Ok(CartridgeType::Mmm01 {
                ram: false,
                battery: false,
            }),
            0x0d => Ok(CartridgeType::Mmm01 {
                ram: true,
                battery: true,
            }),
            0x0f => Ok(CartridgeType::Mbc3 {
                timer: true,
                ram: false,
                battery: true,
            }),
            0x10 => Ok(CartridgeType::Mbc3 {
                timer: true,
                ram: true,
                battery: true,
            }),
            0x11 => Ok(CartridgeType::Mbc3 {
                timer: false,
                ram: false,
                battery: false,
            }),
            0x12 => Ok(CartridgeType::Mbc3 {
                timer: false,
                ram: true,
                battery: false,
            }),
            0x13 => Ok(CartridgeType::Mbc3 {
                timer: false,
                ram: true,
                battery: true,
            }),
            0x19 => Ok(CartridgeType::Mbc5 {
                rumble: false,
                ram: false,
                battery: false,
            }),
            0x1a => Ok(CartridgeType::Mbc5 {
                rumble: false,
                ram: true,
                battery: false,
            }),
            0x1b => Ok(CartridgeType::Mbc5 {
                rumble: false,
                ram: true,
                battery: true,
            }),
            0x1c => Ok(CartridgeType::Mbc5 {
                rumble: true,
                ram: false,
                battery: false,
            }),
            0x1d => Ok(CartridgeType::Mbc5 {
                rumble: true,
                ram: true,
                battery: false,
            }),
            0x1e => Ok(CartridgeType::Mbc5 {
                rumble: true,
                ram: true,
                battery: true,
            }),
            0x20 => Ok(CartridgeType::Mbc6),
            0x22 => Ok(CartridgeType::Mbc7 {
                sensor: true,
                rumble: true,
                ram: true,
                battery: true,
            }),
            0xfc => Ok(CartridgeType::PocketCamera),
            0xfe => Ok(CartridgeType::HuC3),
            0xff => Ok(CartridgeType::HuC1 {
                ram: true,
                battery: true,
            }),
            value => Err(Error::CartridgeType(value)),
        }
    }
}

/// A type specifying general categories of [`Header`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("header missing")]
    Missing,
    #[error("could not parse title")]
    Title(#[from] Utf8Error),
    #[error("invalid CGB flag: {0}")]
    CgbFlag(u8),
    #[error("invalid SGB flag: {0}")]
    SgbFlag(u8),
    #[error("unknown cartridge type: {0:#04x}")]
    CartridgeType(u8),
    #[error("invalid ROM size: {0}")]
    RomSize(u8),
    #[error("invalid RAM size: {0}")]
    RamSize(u8),
    #[error("invalid destination code: {0:#04x}")]
    DestinationCode(u8),
    #[error("bad header checksum (found {found:#04x}, expected {expected:#04x})")]
    HeaderChecksum { found: u8, expected: u8 },
    #[error("bad global checksum: (found {found:#06x}, (expected {expected:#06x})")]
    GlobalChecksum { found: u8, expected: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        let rom: [u8; 0x150] = [
            0xc3, 0x8b, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc3, 0x8b, 0x02, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x87, 0xe1,
            0x5f, 0x16, 0x00, 0x19, 0x5e, 0x23, 0x56, 0xd5, 0xe1, 0xe9, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0xfd, 0x01, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xc3, 0x12, 0x27, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0x12, 0x27, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xc3, 0x7e, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0x00, 0xc3, 0x50, 0x01, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d,
            0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f,
            0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb,
            0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x01, 0x00, 0x00, 0xdc, 0x31, 0xbb,
        ];
        assert_eq!(
            Header::try_from(&rom[..]).unwrap(),
            Header {
                logo: true,
                title: "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_string(),
                dmg: true,
                cgb: false,
                sgb: false,
                cart: CartridgeType::NoMbc {
                    ram: true,
                    battery: false
                },
                romsz: 0x8000,
                ramsz: 0x2000,
                jpn: false,
                version: 0,
                hchk: 0xdc,
                gchk: 0x31bb,
            }
        );
    }
}
