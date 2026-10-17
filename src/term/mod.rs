//! crossterm-based terminal lifecycle, input and output.
//!
//! This is the only module that talks to crossterm. The UI layer produces a
//! [`Frame`]; this layer paints it and turns terminal events into the
//! terminal-independent [`InputEvent`] values consumed by `run`.

mod keys;
mod paint;
mod signals;

use std::io::{self, IsTerminal, Write};
use std::sync::Once;
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event;
use crossterm::execute;
use crossterm::style::{Attribute, SetAttribute};
use crossterm::terminal::{
    self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use crate::input::InputEvent;
use crate::ui::Frame;

/// RAII terminal session. Dropping it restores the terminal.
pub struct Terminal {
    buffer: Vec<u8>,
}

impl Terminal {
    /// Enter raw mode + the alternate screen and hide the cursor.
    pub fn init() -> io::Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(io::Error::other(
                "randloom requires an interactive terminal (stdin/stdout is not a TTY)",
            ));
        }

        terminal::enable_raw_mode()?;
        if let Err(err) = execute!(io::stdout(), EnterAlternateScreen, DisableLineWrap, Hide) {
            restore();
            return Err(err);
        }

        install_panic_hook();
        signals::install();

        Ok(Self {
            buffer: Vec::with_capacity(16 * 1024),
        })
    }

    pub fn size(&self) -> (u16, u16) {
        size()
    }

    /// Clear the visible screen (used after a resize).
    pub fn clear(&mut self) -> io::Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))
    }

    /// Encode and write one full frame.
    pub fn present(&mut self, frame: &Frame) -> io::Result<()> {
        paint::encode(frame, &mut self.buffer)?;
        let mut stdout = io::stdout().lock();
        stdout.write_all(&self.buffer)?;
        stdout.flush()
    }

    /// Wait up to `timeout` for the next key/resize/interrupt event.
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<InputEvent>> {
        if signals::interrupted() {
            return Ok(Some(InputEvent::Interrupt));
        }

        match event::poll(timeout) {
            Ok(false) => Ok(if signals::interrupted() {
                Some(InputEvent::Interrupt)
            } else {
                None
            }),
            Ok(true) => match event::read()? {
                event::Event::Key(key) => Ok(keys::map(key).map(InputEvent::Key)),
                event::Event::Resize(cols, rows) => Ok(Some(InputEvent::Resize(cols, rows))),
                _ => Ok(None),
            },
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok(Some(InputEvent::Interrupt)),
            Err(err) => Err(err),
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}

/// Current terminal size, with the same 80x24 fallback as the C build.
pub fn size() -> (u16, u16) {
    terminal::size().unwrap_or((80, 24))
}

/// Best-effort restore, safe to call more than once (normal exit, panic hook,
/// and Drop can all reach it).
pub fn restore() {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetAttribute(Attribute::Reset),
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    );
    let _ = stdout.flush();
    let _ = terminal::disable_raw_mode();
}

/// Install a panic hook that restores the terminal before the default hook
/// prints the panic message and backtrace.
pub fn install_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));
    });
}

/// Whether a graceful interrupt was requested.
pub fn interrupted() -> bool {
    signals::interrupted()
}
