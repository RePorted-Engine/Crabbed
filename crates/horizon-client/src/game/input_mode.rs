//! Active input chooses only touch overlays; normal menus and the game HUD are unaffected.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
enum Device {
    #[default]
    Touch,
    Keyboard,
    Mouse,
}

#[derive(Default)]
pub(crate) struct InputMode(Device);

impl InputMode {
    pub(crate) fn touch(&mut self) {
        self.0 = Device::Touch;
    }
    pub(crate) fn keyboard(&mut self) {
        self.0 = Device::Keyboard;
    }
    pub(crate) fn mouse(&mut self) {
        self.0 = Device::Mouse;
    }

    pub(crate) fn hides_touch(&self, keyboard: bool, mouse: bool) -> bool {
        match self.0 {
            Device::Touch => false,
            Device::Keyboard => keyboard,
            Device::Mouse => mouse,
        }
    }

    pub(crate) fn refresh(&mut self, keyboard: bool, mouse: bool) {
        if !self.hides_touch(keyboard, mouse) {
            self.touch();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connected_hardware_does_not_hide_touch_until_used_and_touch_restores_it() {
        let mut mode = InputMode::default();
        assert!(!mode.hides_touch(true, true));
        mode.keyboard();
        assert!(mode.hides_touch(true, true));
        mode.mouse();
        assert!(mode.hides_touch(true, true));
        mode.touch();
        assert!(!mode.hides_touch(true, true));
    }
    #[test]
    fn disconnect_and_reconnect_require_new_hardware_activity() {
        let mut mode = InputMode::default();
        mode.mouse();
        mode.refresh(true, false);
        assert!(
            !mode.hides_touch(true, true),
            "keyboard still attached; mouse reconnect alone is not use"
        );
        mode.keyboard();
        mode.refresh(false, true);
        assert!(!mode.hides_touch(true, true));
        mode.keyboard();
        assert!(mode.hides_touch(true, false));
    }
}
