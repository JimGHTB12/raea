//! Bevy's keys as Rune's: a `KeyboardInput` (winit's logical key and the text it typed) as the `KeyEvent` Rune's own
//! shell would have made of the same winit event. Cmd is Command on macOS and Control elsewhere, as Rune has it.

use bevy::input::keyboard::{Key as BKey, KeyboardInput};
use bevy::prelude::*;
use rune::{Key, KeyEvent, Mods};

/// The modifiers held.
pub fn mods(held: &ButtonInput<KeyCode>) -> Mods {
    let any = |a: KeyCode, b: KeyCode| held.pressed(a) || held.pressed(b);
    let logo = any(KeyCode::SuperLeft, KeyCode::SuperRight);
    let ctrl = any(KeyCode::ControlLeft, KeyCode::ControlRight);
    Mods { shift: any(KeyCode::ShiftLeft, KeyCode::ShiftRight), cmd: if cfg!(target_os = "macos") { logo } else { ctrl }, alt: any(KeyCode::AltLeft, KeyCode::AltRight), ctrl, logo }
}

/// `k` as Rune's key event, or none for a key Rune has no name for.
pub fn event(k: &KeyboardInput, mods: Mods) -> Option<KeyEvent> {
    let key = match &k.logical_key {
        BKey::Enter => Key::Enter,
        BKey::Backspace => Key::Backspace,
        BKey::Delete => Key::Delete,
        BKey::ArrowLeft => Key::Left,
        BKey::ArrowRight => Key::Right,
        BKey::ArrowUp => Key::Up,
        BKey::ArrowDown => Key::Down,
        BKey::Home => Key::Home,
        BKey::End => Key::End,
        BKey::PageUp => Key::PageUp,
        BKey::PageDown => Key::PageDown,
        BKey::Insert => Key::Insert,
        BKey::PrintScreen => Key::PrintScreen,
        BKey::ScrollLock => Key::ScrollLock,
        BKey::Pause => Key::Pause,
        BKey::Escape => Key::Escape,
        BKey::Tab => Key::Tab,
        BKey::ContextMenu => Key::ContextMenu,
        BKey::F1 => Key::Function(1),
        BKey::F2 => Key::Function(2),
        BKey::F3 => Key::Function(3),
        BKey::F4 => Key::Function(4),
        BKey::F5 => Key::Function(5),
        BKey::F6 => Key::Function(6),
        BKey::F7 => Key::Function(7),
        BKey::F8 => Key::Function(8),
        BKey::F9 => Key::Function(9),
        BKey::F10 => Key::Function(10),
        BKey::F11 => Key::Function(11),
        BKey::F12 => Key::Function(12),
        // Cmd-Space comes natively as Text(" ") with mods.cmd (rune's KeyEvent::is_command)
        BKey::Space => Key::Text(" ".into()),
        BKey::Character(c) => {
            let ch = c.chars().next()?;
            if mods.cmd { Key::Command(ch.to_ascii_lowercase()) } else { Key::Text(k.text.as_deref().unwrap_or(c).to_string()) }
        }
        _ => return None,
    };
    Some(KeyEvent { key, mods })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: BKey, text: Option<&str>) -> KeyboardInput {
        KeyboardInput { key_code: KeyCode::KeyA, logical_key: key, state: bevy::input::ButtonState::Pressed, text: text.map(Into::into), repeat: false, window: Entity::PLACEHOLDER }
    }

    #[test]
    fn text_named_and_command_keys() {
        let none = Mods::default();
        assert_eq!(event(&press(BKey::Character("a".into()), Some("a")), none).unwrap().key, Key::Text("a".into()));
        assert_eq!(event(&press(BKey::ArrowLeft, None), none).unwrap().key, Key::Left);
        let cmd = Mods { cmd: true, ..none };
        assert_eq!(event(&press(BKey::Character("C".into()), Some("C")), cmd).unwrap().key, Key::Command('c'));
        assert!(event(&press(BKey::Shift, None), none).is_none());
    }
}
