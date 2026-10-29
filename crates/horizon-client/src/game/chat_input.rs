//! Text callbacks from software keyboards have no physical key. Route only
//! their presses as text; coded hardware keys already type via `chat_key_code`.
use winit::{event::ElementState, keyboard::PhysicalKey};

pub(super) fn text_without_physical_key(
    physical_key: PhysicalKey,
    state: ElementState,
    text: Option<&str>,
) -> Option<&str> {
    if state == ElementState::Pressed && matches!(physical_key, PhysicalKey::Unidentified(_)) {
        text
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{KeyCode, NativeKeyCode};

    #[test]
    fn software_text_is_typed_once_and_hardware_text_keeps_its_existing_path() {
        let software = PhysicalKey::Unidentified(NativeKeyCode::Unidentified);
        assert_eq!(
            text_without_physical_key(software, ElementState::Pressed, Some("日本語🦀")),
            Some("日本語🦀")
        );
        assert_eq!(
            text_without_physical_key(software, ElementState::Released, Some("日本語🦀")),
            None
        );
        assert_eq!(
            text_without_physical_key(software, ElementState::Pressed, None),
            None
        );
        for key in [KeyCode::KeyA, KeyCode::ArrowUp, KeyCode::Enter] {
            assert_eq!(
                text_without_physical_key(PhysicalKey::Code(key), ElementState::Pressed, Some("a")),
                None
            );
        }
    }
}
