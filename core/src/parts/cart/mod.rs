//! Game ROM cartridge.
//!
//! Encoded within the ROM is a [header] describing the physical cartridge
//! hardware upon which the ROM resides.
//!
//! Additionally, one of several supported [memory bank controllers][mbcs] may
//! be used to expand the ROM and external RAM beyond the respective 32 KiB and
//! 8 KiB addressable bytes.
//!
//! [header]: https://gbdev.io/pandocs/The_Cartridge_Header.html
//! [mbcs]:   https://gbdev.io/pandocs/MBCs.html

use remus::mio::{Bus, Mmio};
use remus::{Block, Byte};
use thiserror::Error;

use self::header::{Header, Info};
use self::mbc::{Bare, Body};
use crate::api::cart::Cartridge as Api;

pub mod header;
pub mod mbc;

/// Game cartridge.
///
/// Parses a [`Header`] from the ROM, then initializes the memory bank
/// controller ([`mbc`]).
#[derive(Debug)]
pub struct Cartridge {
    /// Cartridge header.
    head: Header,
    /// Cartridge body.
    body: Body,
}

impl Cartridge {
    /// Constructs a new `Cartridge`.
    ///
    /// # Errors
    ///
    /// Returns an error when the cartridge header cannot be parsed.
    pub fn new(rom: &[Byte]) -> Result<Self> {
        let head = Header::new(rom)?;
        Ok(Self {
            body: Body::new(&head, rom)?,
            head,
        })
    }

    /// Constructs a new `Cartridge` explicitly checking the entire header.
    ///
    /// # Errors
    ///
    /// Returns an error when the cartridge header contained an error.
    pub fn checked(rom: &[Byte]) -> Result<Self> {
        // Check then parse cartridge header
        let head = Header::check(rom).and_then(|()| Header::try_from(rom))?;

        // Construct memory bank controller
        let body = Body::new(&head, rom)?;

        Ok(Self { head, body })
    }

    /// Constructs a new `Cartridge` without checking the header.
    ///
    /// # Panics
    ///
    /// Panics if the memory bank controller could not be constructed.
    pub fn unchecked(rom: &[Byte]) -> Self {
        // Parse cartridge header
        let head = Header::new(rom).ok().unwrap_or_else(Header::blank);

        // Construct memory bank controller
        let body = Body::new(&head, rom).ok().unwrap();

        Self { head, body }
    }

    /// Constructs a blank `Cartridge`.
    #[must_use]
    pub fn blank() -> Self {
        // Construct a blank header
        let head = Header::blank();

        // Construct a membory bank controller
        let body = Body::Bare(Bare::new(Box::default(), Box::default()));

        Self { head, body }
    }

    /// Gets the cartridge's title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.head.title.as_deref().unwrap_or("Unknown")
    }

    /// Gets the cartridge's header.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.head
    }

    /// Gets the cartridge's body.
    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Mutably gets the cartridge's body.
    #[must_use]
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

impl Api for Cartridge {}

impl Block for Cartridge {
    fn reset(&mut self) {
        self.body.reset();
    }
}

impl Mmio for Cartridge {
    fn attach(&self, bus: &mut Bus) {
        self.body.attach(bus);
    }

    fn detach(&self, bus: &mut Bus) {
        self.body.detach(bus);
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error caused by constructing a [cartridge](Cartridge).
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to parse header.
    #[error("failed to parse header")]
    Header(#[from] header::Error),
    /// Unsupported cartridge kind.
    #[error("unsupported cartridge: {0}")]
    Unsupported(Info),
}
