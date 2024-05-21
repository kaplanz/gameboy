//! Game ROM cartridge header.
//!
//! Encoded in the ROM at the address range `[$0100..$0150]` is the header, which
//! encodes both physical attributes describing the hardware of the cartridge,
//! flags describing console support, and characteristics of the software.
//!
//! See more details [here][header].
//!
//! [header]: https://gbdev.io/pandocs/The_Cartridge_Header.html

use std::array::TryFromSliceError;
use std::fmt::Display;
use std::str::Utf8Error;

use log::{error, warn};
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

/// Calculates the header checksum.
#[must_use]
pub fn hchk(rom: &[Byte]) -> Byte {
    rom[0x134..=0x14c]
        .iter()
        .copied()
        .fold(0, |accum, item| accum.wrapping_sub(item).wrapping_sub(1))
}

/// Calculates the global checksum.
#[must_use]
pub fn gchk(rom: &[Byte]) -> Word {
    rom.iter()
        .copied()
        .fold(0u16, |accum, item| accum.wrapping_add(Word::from(item)))
        .wrapping_sub(Word::from(rom[0x14e]))
        .wrapping_sub(Word::from(rom[0x14f]))
}

/// Cartridge header.
///
/// Information about the ROM and the cartridge containing it. Stored in the
/// address range `[$0100, $0150)`.
#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Header {
    /// `[$0104..=$0133]`: Nintendo logo.
    ///
    /// Check for if the cartridge was officially licensed by Nintendo.
    /// Cartridges must include the [expected bytes](LOGO) to render Nintendo's
    /// logo in the boot ROM.
    ///
    /// See more details [here][logo].
    ///
    /// [logo]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0104-0133--nintendo-logo
    pub logo: bool,
    /// `[$0134..=$0143]`: Title.
    ///
    /// The title of the game in uppercase ASCII.
    ///
    /// See more details [here][title].
    ///
    /// [title]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0134-0143--title
    pub title: Option<String>,
    /// `[$0143]`: DMG flag.
    ///
    /// Whether this cartridge is compatible with the DMG model.
    ///
    /// See more details [here][dmg].
    ///
    /// [dmg]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0143--cgb-flag
    pub dmg: bool,
    /// `[$0143]`: CGB flag.
    ///
    /// Whether this cartridge is compatible with the CGB model.
    ///
    /// See more details [here][cgb].
    ///
    /// [cgb]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0143--cgb-flag
    pub cgb: bool,
    /// `[$0146]`: SGB flag.
    ///
    /// Whether this cartridge is compatible with the SGB model.
    ///
    /// See more details [here][sgb].
    ///
    /// [sgb]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0146--sgb-flag
    pub sgb: bool,
    /// `[$0147]`: Cartridge type.
    ///
    /// Whether this cartridge is compatible with the SGB model.
    ///
    /// See more details [here][type].
    ///
    /// [type]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
    pub info: Info,
    /// `[$0148]`: ROM size.
    ///
    /// How much ROM is present on the cartridge.
    ///
    /// See more details [here][romsz].
    ///
    /// [romsz]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0148--rom-size
    pub romsz: usize,
    /// `[$0149]`: RAM size.
    ///
    /// How much RAM is present on the cartridge, if any.
    ///
    /// See more details [here][ramsz].
    ///
    /// [ramsz]: https://gbdev.io/pandocs/The_Cartridge_Header.html#0149--ram-size
    pub ramsz: usize,
    /// `[$014A]`: Destination code.
    ///
    /// Whether this version of the game is intended to be sold in Japan or
    /// elsewhere.
    ///
    /// See more details [here][jpn].
    ///
    /// [jpn]: https://gbdev.io/pandocs/The_Cartridge_Header.html#014a--destination-code
    pub jpn: bool,
    /// `[$014C]`: Mask ROM version number.
    ///
    /// Version number of the game.
    ///
    /// See more details [here][version].
    ///
    /// [version]: https://gbdev.io/pandocs/The_Cartridge_Header.html#014c--mask-rom-version-number
    pub version: Byte,
    /// `[$014D]`: Header checksum.
    ///
    /// An 8-bit checksum computed from the cartridge header bytes
    /// `[$0134..=$014C]`.
    ///
    /// See more details [here][hchk].
    ///
    /// [hchk]: https://gbdev.io/pandocs/The_Cartridge_Header.html#014d--header-checksum
    pub hchk: Byte,
    /// `[$014E..=$014F]`: Global checksum.
    ///
    /// A 16-bit checksum computed from the entire cartridge ROM (excluding
    /// these two bytes). This checksum is not usually verified.
    ///
    /// See more details [here][gchk].
    ///
    /// [gchk]: https://gbdev.io/pandocs/The_Cartridge_Header.html#014e-014f--global-checksum
    pub gchk: Word,
}

impl Header {
    /// Constructs a new `Header`.
    ///
    /// # Errors
    ///
    /// Returns an error if the ROM contained invalid header bytes.
    pub fn new(rom: &[Byte]) -> Result<Self> {
        // Extract header bytes
        let head: &[Byte; 0x50] = rom.get(0x100..0x150).ok_or(Error::Missing)?.try_into()?;

        // Construct header
        let this = Self {
            // Compare logo data
            logo: make::logo(head),
            // Parse title
            title: make::title(head)?,
            // Parse DMG flag
            dmg: make::dmg(head),
            // Parse CGB flag
            cgb: make::cgb(head)?,
            // Parse SGB flag
            sgb: make::sgb(head),
            // Parse cartridge type
            info: make::info(head)?,
            // Parse ROM size
            romsz: make::romsz(head)?,
            // Parse RAM size
            ramsz: make::ramsz(head)?,
            // Parse RAM size
            jpn: make::jpn(head)?,
            // Parse version number
            version: make::version(head),
            // Parse header checksum
            hchk: make::hchk(head),
            // Parse global checksum
            gchk: make::gchk(head),
        };

        // Validate header checksum
        let chk = self::hchk(rom);
        if chk != this.hchk {
            return Err(Error::HeaderChk {
                found: chk,
                expected: this.hchk,
            });
        }
        // Validate global checksum
        let chk = self::gchk(rom);
        if chk != this.gchk {
            warn!(
                "bad global checksum (found {found:#06x}, expected {expected:#06x})",
                found = chk,
                expected = this.gchk,
            );
        }

        Ok(this)
    }

    /// Constructs a new `Header`, checking cartridge integrity.
    ///
    /// # Errors
    ///
    /// Returns an error if the ROM contained invalid header bytes, or if
    /// cartridge integrity seems compromised. (This is detected using
    /// checksums.)
    pub fn checked(rom: &[Byte]) -> Result<Self> {
        // Extract header bytes
        let head: &[Byte; 0x50] = rom.get(0x100..0x150).ok_or(Error::Missing)?.try_into()?;

        // Construct header
        let this = Self {
            // Compare logo data
            logo: make::logo(head),
            // Parse title
            title: make::title(head)?,
            // Parse DMG flag
            dmg: make::dmg(head),
            // Parse CGB flag
            cgb: make::cgb(head)?,
            // Parse SGB flag
            sgb: make::sgb(head),
            // Parse cartridge type
            info: make::info(head)?,
            // Parse ROM size
            romsz: make::romsz(head)?,
            // Parse RAM size
            ramsz: make::ramsz(head)?,
            // Parse RAM size
            jpn: make::jpn(head)?,
            // Parse version number
            version: make::version(head),
            // Parse header checksum
            hchk: make::hchk(head),
            // Parse global checksum
            gchk: make::gchk(head),
        };

        // Validate header checksum
        let chk = self::hchk(rom);
        if chk != this.hchk {
            return Err(Error::HeaderChk {
                found: chk,
                expected: this.hchk,
            });
        }
        // Validate global checksum
        let chk = self::gchk(rom);
        if chk != this.gchk {
            return Err(Error::GlobalChk {
                found: chk,
                expected: this.gchk,
            });
        }

        Ok(this)
    }

    /// Constructs a new `Header`, ignoring invalid header bytes.
    ///
    /// # Errors
    ///
    /// Header construction will fail if the ROM is missing header bytes.
    pub fn unchecked(rom: &[Byte]) -> Result<Self> {
        // Extract header bytes
        let head: &[Byte; 0x50] = rom.get(0x100..0x150).ok_or(Error::Missing)?.try_into()?;

        // Construct header
        let ramsz = make::ramsz(head);
        let this = Self {
            // Compare logo data
            logo: make::logo(head),
            // Parse title
            title: make::title(head).unwrap_or_else(|err| {
                let default = Option::default();
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse DMG flag
            dmg: make::dmg(head),
            // Parse CGB flag
            cgb: make::cgb(head).unwrap_or_else(|err| {
                let default = bool::default();
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse SGB flag
            sgb: make::sgb(head),
            // Parse cartridge type
            info: make::info(head).unwrap_or_else(|err| {
                let default = Info::Bare {
                    ram: matches!(ramsz, Ok(ramsz) if ramsz > 0),
                    pwr: false,
                };
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse ROM size
            romsz: make::romsz(head).unwrap_or_else(|err| {
                let default = 0x8000;
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse RAM size
            ramsz: ramsz.unwrap_or_else(|err| {
                let default = usize::default();
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse RAM size
            jpn: make::jpn(head).unwrap_or_else(|err| {
                let default = bool::default();
                error!("{err} (default: {default:?})");
                default
            }),
            // Parse version number
            version: make::version(head),
            // Parse header checksum
            hchk: make::hchk(head),
            // Parse global checksum
            gchk: make::gchk(head),
        };

        Ok(this)
    }
}

mod make {
    use log::warn;
    use remus::{Byte, Word};

    use super::{Error, Info, Result, LOGO};

    /// Parse the `logo` field from the header.
    pub fn logo(head: &[Byte; 0x50]) -> bool {
        head[0x04..=0x33] == LOGO
    }

    /// Parse the `title` field from the header.
    pub fn title(head: &[Byte; 0x50]) -> Result<Option<String>> {
        let tlen = if head[0x43] & 0x80 == 0 { 16 } else { 15 };
        let title = std::str::from_utf8(&head[0x34..0x34 + tlen])
            .map_err(Error::Title)?
            .trim_matches('\0')
            .to_string();
        Ok(if title.is_empty() { None } else { Some(title) })
    }

    /// Parse the `dmg` field from the header.
    pub fn dmg(head: &[Byte; 0x50]) -> bool {
        (head[0x43] & 0xc0) != 0xc0
    }

    /// Parse the `cgb` field from the header.
    pub fn cgb(head: &[Byte; 0x50]) -> Result<bool> {
        match head[0x43] & 0xbf {
            0x00 => Ok(false),
            0x80 => Ok(true),
            byte => Err(Error::Color(byte)),
        }
    }

    /// Parse the `sgb` field from the header.
    pub fn sgb(head: &[Byte; 0x50]) -> bool {
        match head[0x46] {
            0x00 => false,
            0x03 => true,
            byte => {
                warn!("non-standard SGB flag: {byte:#04x}");
                false
            }
        }
    }

    /// Parse the `info` field from the header.
    pub fn info(head: &[Byte; 0x50]) -> Result<Info> {
        head[0x47].try_into()
    }

    /// Parse the `romsz` field from the header.
    pub fn romsz(head: &[Byte; 0x50]) -> Result<usize> {
        match head[0x48] {
            byte @ 0x00..=0x08 => Ok(0x8000 << byte),
            byte => Err(Error::Rom(byte)),
        }
    }

    /// Parse the `ramsz` field from the header.
    pub fn ramsz(head: &[Byte; 0x50]) -> Result<usize> {
        match head[0x49] {
            0x00 => Ok(0x00000),
            0x01 => Ok(0x00800),
            0x02 => Ok(0x02000),
            0x03 => Ok(0x08000),
            0x04 => Ok(0x20000),
            0x05 => Ok(0x10000),
            byte => Err(Error::Ram(byte)),
        }
    }

    /// Parse the `jpn` field from the header.
    pub fn jpn(head: &[Byte; 0x50]) -> Result<bool> {
        match head[0x4a] {
            byte @ (0x00 | 0x01) => Ok(byte == 0x00),
            byte => Err(Error::Region(byte)),
        }
    }

    /// Parse the `version` field from the header.
    pub fn version(head: &[Byte; 0x50]) -> Byte {
        head[0x4c]
    }

    /// Parse the `hchk` field from the header.
    pub fn hchk(head: &[Byte; 0x50]) -> Byte {
        head[0x4d]
    }

    /// Parse the `gchk` field from the header.
    pub fn gchk(head: &[Byte; 0x50]) -> Word {
        Word::from_be_bytes([head[0x4e], head[0x4f]])
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
