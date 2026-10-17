//! Terminal-independent input events and application flow control.

/// A key the TUI reacts to.
///
/// Printable keys that the application does not use are delivered as
/// [`Key::Char`] and ignored by the event handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Tab,
    Backspace,
    Enter,
    Esc,
    CtrlC,
}

/// An event delivered by the terminal layer to the run loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Key(Key),
    Resize(u16, u16),
    Interrupt,
}

/// Whether the application should keep running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}
