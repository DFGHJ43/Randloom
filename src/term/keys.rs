//! Map crossterm key events to the terminal-independent [`Key`] type.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::input::Key;

/// Ignore key releases; repeated key presses (auto-repeat) are treated like
/// normal presses, which matches the C `_getch`/ANSI loop.
pub fn map(event: KeyEvent) -> Option<Key> {
    if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);

    match event.code {
        KeyCode::Char('c') if ctrl => Some(Key::CtrlC),
        KeyCode::Char(c) if !ctrl && !alt => Some(Key::Char(c)),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Left => Some(Key::Left),
        KeyCode::Right => Some(Key::Right),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Esc => Some(Key::Esc),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn maps_application_keys() {
        assert_eq!(
            map(press(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(Key::Char('q'))
        );
        assert_eq!(
            map(press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Key::CtrlC)
        );
        assert_eq!(map(press(KeyCode::Up, KeyModifiers::NONE)), Some(Key::Up));
        assert_eq!(map(press(KeyCode::Tab, KeyModifiers::NONE)), Some(Key::Tab));
        assert_eq!(
            map(press(KeyCode::Backspace, KeyModifiers::NONE)),
            Some(Key::Backspace)
        );
        assert_eq!(map(press(KeyCode::Esc, KeyModifiers::NONE)), Some(Key::Esc));
    }

    #[test]
    fn ignores_release_and_control_character_typing() {
        let mut event = press(KeyCode::Char('x'), KeyModifiers::NONE);
        event.kind = KeyEventKind::Release;
        assert_eq!(map(event), None);

        assert_eq!(map(press(KeyCode::Char('u'), KeyModifiers::CONTROL)), None);
    }
}
