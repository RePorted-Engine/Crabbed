//! UIKit UIKeyboardHIDUsage values are USB HID keyboard usages, independent of keyboard layout.
use crate::keyboard::{Key, KeyCode, KeyLocation, NamedKey, NativeKey, NativeKeyCode, PhysicalKey};

/// UIKit may spell navigation keys as `UIKeyInputUpArrow` instead of a
/// control character. Named keys carry actions, never character input.
pub(super) fn key_text(logical: &Key, text: &str) -> Option<smol_str::SmolStr> {
    (matches!(logical, Key::Character(_))
        && !text.is_empty()
        && text_key_code(text).is_none()
        && !text
            .chars()
            .any(|c| c.is_control() || ('\u{f700}'..='\u{f8ff}').contains(&c)))
    .then(|| text.into())
}

/// `UIKeyInput::insertText:` is an independent input path: UIKit can send a
/// navigation token here without delivering a `UIPress`. Decode the whole token
/// before splitting real text into characters, so its name cannot become typing.
pub(super) fn insert_text_keys(text: &str) -> Vec<(PhysicalKey, Key)> {
    if let Some(code) = text_key_code(text) {
        return vec![(
            PhysicalKey::Code(code),
            logical_key(PhysicalKey::Code(code), text),
        )];
    }
    text.chars()
        .filter_map(|c| {
            let text = smol_str::SmolStr::from_iter([c]);
            if let Some(code) = text_key_code(&text) {
                let physical = PhysicalKey::Code(code);
                Some((physical, logical_key(physical, &text)))
            } else {
                let logical = Key::Character(text.clone());
                key_text(&logical, &text).map(|_| {
                    (
                        PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
                        logical,
                    )
                })
            }
        })
        .collect()
}

fn text_key_code(text: &str) -> Option<KeyCode> {
    Some(match text {
        "UIKeyInputUpArrow" | "\u{f700}" => KeyCode::ArrowUp,
        "UIKeyInputDownArrow" | "\u{f701}" => KeyCode::ArrowDown,
        "UIKeyInputLeftArrow" | "\u{f702}" => KeyCode::ArrowLeft,
        "UIKeyInputRightArrow" | "\u{f703}" => KeyCode::ArrowRight,
        "UIKeyInputEscape" | "\u{1b}" => KeyCode::Escape,
        "UIKeyInputTab" | "\t" => KeyCode::Tab,
        "UIKeyInputHome" | "\u{f729}" => KeyCode::Home,
        "UIKeyInputEnd" | "\u{f72b}" => KeyCode::End,
        "UIKeyInputPageUp" | "\u{f72c}" => KeyCode::PageUp,
        "UIKeyInputPageDown" | "\u{f72d}" => KeyCode::PageDown,
        "\n" | "\r" => KeyCode::Enter,
        "\u{8}" | "\u{7f}" => KeyCode::Backspace,
        "\u{f728}" => KeyCode::Delete,
        _ => return None,
    })
}

pub(super) fn physical_key(usage: usize) -> PhysicalKey {
    PhysicalKey::Code(match usage {
        4 => KeyCode::KeyA,
        5 => KeyCode::KeyB,
        6 => KeyCode::KeyC,
        7 => KeyCode::KeyD,
        8 => KeyCode::KeyE,
        9 => KeyCode::KeyF,
        10 => KeyCode::KeyG,
        11 => KeyCode::KeyH,
        12 => KeyCode::KeyI,
        13 => KeyCode::KeyJ,
        14 => KeyCode::KeyK,
        15 => KeyCode::KeyL,
        16 => KeyCode::KeyM,
        17 => KeyCode::KeyN,
        18 => KeyCode::KeyO,
        19 => KeyCode::KeyP,
        20 => KeyCode::KeyQ,
        21 => KeyCode::KeyR,
        22 => KeyCode::KeyS,
        23 => KeyCode::KeyT,
        24 => KeyCode::KeyU,
        25 => KeyCode::KeyV,
        26 => KeyCode::KeyW,
        27 => KeyCode::KeyX,
        28 => KeyCode::KeyY,
        29 => KeyCode::KeyZ,
        30 => KeyCode::Digit1,
        31 => KeyCode::Digit2,
        32 => KeyCode::Digit3,
        33 => KeyCode::Digit4,
        34 => KeyCode::Digit5,
        35 => KeyCode::Digit6,
        36 => KeyCode::Digit7,
        37 => KeyCode::Digit8,
        38 => KeyCode::Digit9,
        39 => KeyCode::Digit0,
        40 => KeyCode::Enter,
        41 => KeyCode::Escape,
        42 => KeyCode::Backspace,
        43 => KeyCode::Tab,
        44 => KeyCode::Space,
        45 => KeyCode::Minus,
        46 => KeyCode::Equal,
        47 => KeyCode::BracketLeft,
        48 => KeyCode::BracketRight,
        49 => KeyCode::Backslash,
        50 => KeyCode::Backslash,
        51 => KeyCode::Semicolon,
        52 => KeyCode::Quote,
        53 => KeyCode::Backquote,
        54 => KeyCode::Comma,
        55 => KeyCode::Period,
        56 => KeyCode::Slash,
        57 => KeyCode::CapsLock,
        58 => KeyCode::F1,
        59 => KeyCode::F2,
        60 => KeyCode::F3,
        61 => KeyCode::F4,
        62 => KeyCode::F5,
        63 => KeyCode::F6,
        64 => KeyCode::F7,
        65 => KeyCode::F8,
        66 => KeyCode::F9,
        67 => KeyCode::F10,
        68 => KeyCode::F11,
        69 => KeyCode::F12,
        70 => KeyCode::PrintScreen,
        71 => KeyCode::ScrollLock,
        72 => KeyCode::Pause,
        73 => KeyCode::Insert,
        74 => KeyCode::Home,
        75 => KeyCode::PageUp,
        76 => KeyCode::Delete,
        77 => KeyCode::End,
        78 => KeyCode::PageDown,
        79 => KeyCode::ArrowRight,
        80 => KeyCode::ArrowLeft,
        81 => KeyCode::ArrowDown,
        82 => KeyCode::ArrowUp,
        83 => KeyCode::NumLock,
        84 => KeyCode::NumpadDivide,
        85 => KeyCode::NumpadMultiply,
        86 => KeyCode::NumpadSubtract,
        87 => KeyCode::NumpadAdd,
        88 => KeyCode::NumpadEnter,
        89 => KeyCode::Numpad1,
        90 => KeyCode::Numpad2,
        91 => KeyCode::Numpad3,
        92 => KeyCode::Numpad4,
        93 => KeyCode::Numpad5,
        94 => KeyCode::Numpad6,
        95 => KeyCode::Numpad7,
        96 => KeyCode::Numpad8,
        97 => KeyCode::Numpad9,
        98 => KeyCode::Numpad0,
        99 => KeyCode::NumpadDecimal,
        100 => KeyCode::IntlBackslash,
        101 => KeyCode::ContextMenu,
        103 => KeyCode::NumpadEqual,
        224 => KeyCode::ControlLeft,
        225 => KeyCode::ShiftLeft,
        226 => KeyCode::AltLeft,
        227 => KeyCode::SuperLeft,
        228 => KeyCode::ControlRight,
        229 => KeyCode::ShiftRight,
        230 => KeyCode::AltRight,
        231 => KeyCode::SuperRight,
        _ => return PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
    })
}

pub(super) fn location(key: PhysicalKey) -> KeyLocation {
    match key {
        PhysicalKey::Code(
            KeyCode::ShiftLeft | KeyCode::ControlLeft | KeyCode::AltLeft | KeyCode::SuperLeft,
        ) => KeyLocation::Left,
        PhysicalKey::Code(
            KeyCode::ShiftRight | KeyCode::ControlRight | KeyCode::AltRight | KeyCode::SuperRight,
        ) => KeyLocation::Right,
        PhysicalKey::Code(
            KeyCode::NumpadDivide
            | KeyCode::NumpadMultiply
            | KeyCode::NumpadSubtract
            | KeyCode::NumpadAdd
            | KeyCode::NumpadEnter
            | KeyCode::Numpad1
            | KeyCode::Numpad2
            | KeyCode::Numpad3
            | KeyCode::Numpad4
            | KeyCode::Numpad5
            | KeyCode::Numpad6
            | KeyCode::Numpad7
            | KeyCode::Numpad8
            | KeyCode::Numpad9
            | KeyCode::Numpad0
            | KeyCode::NumpadDecimal
            | KeyCode::NumpadEqual,
        ) => KeyLocation::Numpad,
        _ => KeyLocation::Standard,
    }
}

pub(super) fn logical_key(key: PhysicalKey, text: &str) -> Key {
    let named = match key {
        PhysicalKey::Code(KeyCode::Enter) => NamedKey::Enter,
        PhysicalKey::Code(KeyCode::Escape) => NamedKey::Escape,
        PhysicalKey::Code(KeyCode::Backspace) => NamedKey::Backspace,
        PhysicalKey::Code(KeyCode::Tab) => NamedKey::Tab,
        PhysicalKey::Code(KeyCode::CapsLock) => NamedKey::CapsLock,
        PhysicalKey::Code(KeyCode::Insert) => NamedKey::Insert,
        PhysicalKey::Code(KeyCode::Home) => NamedKey::Home,
        PhysicalKey::Code(KeyCode::End) => NamedKey::End,
        PhysicalKey::Code(KeyCode::Delete) => NamedKey::Delete,
        PhysicalKey::Code(KeyCode::PageUp) => NamedKey::PageUp,
        PhysicalKey::Code(KeyCode::PageDown) => NamedKey::PageDown,
        PhysicalKey::Code(KeyCode::ArrowLeft) => NamedKey::ArrowLeft,
        PhysicalKey::Code(KeyCode::ArrowRight) => NamedKey::ArrowRight,
        PhysicalKey::Code(KeyCode::ArrowUp) => NamedKey::ArrowUp,
        PhysicalKey::Code(KeyCode::ArrowDown) => NamedKey::ArrowDown,
        PhysicalKey::Code(KeyCode::NumLock) => NamedKey::NumLock,
        PhysicalKey::Code(KeyCode::ScrollLock) => NamedKey::ScrollLock,
        PhysicalKey::Code(KeyCode::Pause) => NamedKey::Pause,
        PhysicalKey::Code(KeyCode::PrintScreen) => NamedKey::PrintScreen,
        PhysicalKey::Code(KeyCode::F1) => NamedKey::F1,
        PhysicalKey::Code(KeyCode::F2) => NamedKey::F2,
        PhysicalKey::Code(KeyCode::F3) => NamedKey::F3,
        PhysicalKey::Code(KeyCode::F4) => NamedKey::F4,
        PhysicalKey::Code(KeyCode::F5) => NamedKey::F5,
        PhysicalKey::Code(KeyCode::F6) => NamedKey::F6,
        PhysicalKey::Code(KeyCode::F7) => NamedKey::F7,
        PhysicalKey::Code(KeyCode::F8) => NamedKey::F8,
        PhysicalKey::Code(KeyCode::F9) => NamedKey::F9,
        PhysicalKey::Code(KeyCode::F10) => NamedKey::F10,
        PhysicalKey::Code(KeyCode::F11) => NamedKey::F11,
        PhysicalKey::Code(KeyCode::F12) => NamedKey::F12,
        PhysicalKey::Code(KeyCode::NumpadEnter) => NamedKey::Enter,
        PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => NamedKey::Shift,
        PhysicalKey::Code(KeyCode::ControlLeft | KeyCode::ControlRight) => NamedKey::Control,
        PhysicalKey::Code(KeyCode::AltLeft | KeyCode::AltRight) => NamedKey::Alt,
        PhysicalKey::Code(KeyCode::SuperLeft | KeyCode::SuperRight) => NamedKey::Super,
        _ => {
            return if text.is_empty() {
                Key::Unidentified(NativeKey::Unidentified)
            } else {
                Key::Character(text.into())
            }
        }
    };
    Key::Named(named)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_keys_keep_actions_without_inserting_uikit_names() {
        for (usage, name, expected) in [
            (82, "UIKeyInputUpArrow", NamedKey::ArrowUp),
            (81, "UIKeyInputDownArrow", NamedKey::ArrowDown),
            (80, "UIKeyInputLeftArrow", NamedKey::ArrowLeft),
            (79, "UIKeyInputRightArrow", NamedKey::ArrowRight),
            (43, "UIKeyInputTab", NamedKey::Tab),
            (74, "UIKeyInputHome", NamedKey::Home),
        ] {
            let logical = logical_key(physical_key(usage), name);
            assert_eq!(logical, Key::Named(expected));
            assert_eq!(key_text(&logical, name), None);
        }
        assert_eq!(key_text(&Key::Named(NamedKey::ArrowUp), "\u{f700}"), None);
        assert_eq!(
            key_text(&logical_key(physical_key(4), "q"), "q").as_deref(),
            Some("q")
        );
        assert_eq!(
            key_text(&logical_key(physical_key(44), " "), " ").as_deref(),
            Some(" ")
        );
    }
    #[test]
    fn insert_text_decodes_navigation_before_splitting_into_characters() {
        for (text, code, named) in [
            ("UIKeyInputUpArrow", KeyCode::ArrowUp, NamedKey::ArrowUp),
            (
                "UIKeyInputDownArrow",
                KeyCode::ArrowDown,
                NamedKey::ArrowDown,
            ),
            (
                "UIKeyInputLeftArrow",
                KeyCode::ArrowLeft,
                NamedKey::ArrowLeft,
            ),
            (
                "UIKeyInputRightArrow",
                KeyCode::ArrowRight,
                NamedKey::ArrowRight,
            ),
            ("UIKeyInputHome", KeyCode::Home, NamedKey::Home),
            ("UIKeyInputEnd", KeyCode::End, NamedKey::End),
            ("UIKeyInputPageUp", KeyCode::PageUp, NamedKey::PageUp),
            ("UIKeyInputPageDown", KeyCode::PageDown, NamedKey::PageDown),
            ("UIKeyInputEscape", KeyCode::Escape, NamedKey::Escape),
            ("UIKeyInputTab", KeyCode::Tab, NamedKey::Tab),
            ("\u{f700}", KeyCode::ArrowUp, NamedKey::ArrowUp),
            ("\u{f701}", KeyCode::ArrowDown, NamedKey::ArrowDown),
            ("\u{f702}", KeyCode::ArrowLeft, NamedKey::ArrowLeft),
            ("\u{f703}", KeyCode::ArrowRight, NamedKey::ArrowRight),
            ("\n", KeyCode::Enter, NamedKey::Enter),
            ("\t", KeyCode::Tab, NamedKey::Tab),
            ("\u{7f}", KeyCode::Backspace, NamedKey::Backspace),
            ("\u{f728}", KeyCode::Delete, NamedKey::Delete),
        ] {
            // The exact conversion called by WinitView::handle_insert_text. One
            // actionable key, no character events for the UIKit token's letters.
            let keys = insert_text_keys(text);
            assert_eq!(keys, vec![(PhysicalKey::Code(code), Key::Named(named))]);
            assert_eq!(key_text(&keys[0].1, text), None);
            // Also reject a token if a hardware keyboard reports an unexpected HID.
            assert_eq!(key_text(&Key::Character(text.into()), text), None);
        }
    }

    #[test]
    fn insert_text_preserves_unicode_and_does_not_strip_token_substrings() {
        for text in [
            "Hello world",
            "é日本語🦀",
            "e\u{301}",
            "say UIKeyInputUpArrow!",
        ] {
            let keys = insert_text_keys(text);
            let typed: String = keys
                .iter()
                .filter_map(|(_, key)| match key {
                    Key::Character(text) => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(typed, text);
        }
        assert!(insert_text_keys("").is_empty());
        assert!(insert_text_keys("\u{0}\u{f8ff}").is_empty());
    }

    #[test]
    fn layout_text_is_independent_of_gameplay_position() {
        let key = physical_key(4);
        assert_eq!(key, PhysicalKey::Code(KeyCode::KeyA));
        assert_eq!(logical_key(key, "q"), Key::Character("q".into()));
        assert_eq!(
            logical_key(physical_key(41), ""),
            Key::Named(NamedKey::Escape)
        );
        assert_eq!(location(physical_key(229)), KeyLocation::Right);
        assert_eq!(location(physical_key(88)), KeyLocation::Numpad);
        assert!(matches!(physical_key(999), PhysicalKey::Unidentified(_)));
    }
}
