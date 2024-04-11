//! Application frontend.

use std::io::{self, Read, Write};
use std::net::UdpSocket as Link;

use log::{debug, trace};
use minifb::Key;
use rugby::app::audio::Audio;
use rugby::app::joypad::Joypad;
use rugby::app::serial::Serial;
use rugby::app::video::Video;
use rugby::core::dmg::{self, Button};
use rugby::emu::joypad::Event;
use rugby::pal::Palette;

mod win;

#[cfg(feature = "win")]
pub use self::win::dbg;
pub use self::win::Graphics;

/// Frontend options.
#[derive(Debug)]
pub struct Options {
    /// Color palette.
    pub pal: Palette,
}

/// Application frontend.
#[derive(Debug)]
pub struct Frontend {
    /// Frontend options.
    pub cfg: Options,
    /// Link cable.
    pub lnk: Option<Link>,
    /// Window graphics.
    pub win: Option<Graphics>,
}

impl Audio for Frontend {}

impl Joypad for Frontend {
    type Button = Button;

    #[rustfmt::skip]
    fn input(&mut self) -> Vec<Event<Self::Button>> {
        self.win
            .as_ref()
            // Fetch keys
            .map(|gui| gui.lcd.keys())
            .into_iter()
            // Remove nested optional
            .flatten()
            // Perform key mapping
            .filter_map(|Event { input: key, state }| match key {
                Key::Z     => Some(Event { input: Button::A,      state }),
                Key::X     => Some(Event { input: Button::B,      state }),
                Key::Space => Some(Event { input: Button::Select, state }),
                Key::Enter => Some(Event { input: Button::Start,  state }),
                Key::Right => Some(Event { input: Button::Right,  state }),
                Key::Left  => Some(Event { input: Button::Left,   state }),
                Key::Up    => Some(Event { input: Button::Up,     state }),
                Key::Down  => Some(Event { input: Button::Down,   state }),
                _ => None,
            }).collect()
    }
}

impl Serial for Frontend {
    fn recv(&mut self, mut tx: impl Read) -> io::Result<usize> {
        // Extract remote link
        let Some(link) = self.lnk.as_mut() else {
            return Ok(0);
        };
        // Download from emulator
        let mut buf = Vec::new();
        let read = tx.read_to_end(&mut buf)?;
        if read == 0 {
            return Ok(0);
        }
        // Transmit data to link
        let sent = link.send(&buf)?;
        debug!("transmitted {sent} bytes ({read} downloaded)");
        trace!("serial tx: {buf:?}");
        Ok(read)
    }

    fn send(&mut self, mut rx: impl Write) -> io::Result<usize> {
        // Extract remote link
        let Some(link) = self.lnk.as_mut() else {
            return Ok(0);
        };
        // Receive data from link
        let mut buf = [0; 0x10]; // use fixed-size buffer
        let recvd = match link.recv(&mut buf) {
            // Explicitly ignore would block error
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(0),
            res => res,
        }?;
        let buf = &buf[..recvd]; // retain only valid data
        if recvd == 0 {
            return Ok(0);
        }
        // Upload to emulator
        let wrote = rx.write(buf)?;
        debug!("received {recvd} bytes ({wrote} uploaded)");
        trace!("serial rx: {buf:?}");
        Ok(wrote)
    }
}

impl Video for Frontend {
    type Pixel = dmg::ppu::Color;

    fn draw(&mut self, frame: &[Self::Pixel]) {
        // Extract GUI
        let Some(gui) = self.win.as_mut() else {
            return;
        };
        // Translate pixels
        let frame = frame
            .iter()
            .map(|&pix| self.cfg.pal[pix as usize])
            .map(u32::from)
            .collect::<Vec<_>>();
        // Redraw main window
        gui.lcd.redraw(&frame).unwrap();
    }
}
