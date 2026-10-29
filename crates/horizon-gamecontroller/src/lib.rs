//! Controllers on iPhone and iPad: Apple's GameController framework, which pairs and reads every pad iOS
//! supports (DualSense, DualShock 4, Xbox, MFi). The game forbids `unsafe`, so the Objective-C calls live
//! here and the game sees plain values. GameController keeps its state current on the main thread's run
//! loop, which is the one winit runs, so reading a pad is reading properties: no event queue to drain.
//! Elsewhere the functions report no controllers.

/// One controller iOS has paired.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Controller {
    /// The vendor's name for it, when it gives one.
    pub name: Option<String>,
    /// iOS's product category: "DualSense", "DualShock 4", "Xbox One", "Switch Pro Controller"...
    pub category: String,
    /// It has the full gamepad profile (two sticks, four face buttons, shoulders and triggers).
    pub gamepad: bool,
}

/// The active pad's inputs: sticks -1..1 with up positive, triggers 0..1.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub category: String,
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    /// The left one of the middle buttons (View, Share, Create, minus); not every pad has it.
    pub options: bool,
    /// The right one (Menu, Options, plus).
    pub menu: bool,
    pub left_stick_button: bool,
    pub right_stick_button: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub left_stick: [f32; 2],
    pub right_stick: [f32; 2],
    pub triggers: [f32; 2],
}

#[cfg(target_os = "ios")]
mod ios {
    use super::{Controller, Snapshot};
    use objc2_game_controller::{
        GCController, GCControllerButtonInput, GCDevice, GCExtendedGamepad,
    };

    pub fn controllers() -> Vec<Controller> {
        // SAFETY: class and property reads of GameController objects, which iOS keeps alive while listed.
        unsafe {
            GCController::controllers()
                .iter()
                .map(|c| Controller {
                    name: c.vendorName().map(|n| n.to_string()),
                    category: c.productCategory().to_string(),
                    gamepad: c.extendedGamepad().is_some(),
                })
                .collect()
        }
    }

    fn pressed(button: Option<objc2::rc::Retained<GCControllerButtonInput>>) -> bool {
        // SAFETY: a property read of a live button.
        button.is_some_and(|b| unsafe { b.isPressed() })
    }

    pub fn active() -> Option<Snapshot> {
        // SAFETY: as above; every object is retained for the length of the read.
        unsafe {
            let gamepad = |c: &GCController| c.extendedGamepad().is_some();
            let controller = GCController::current()
                .filter(|c| gamepad(c))
                .or_else(|| GCController::controllers().iter().find(|c| gamepad(c)))?;
            let pad: objc2::rc::Retained<GCExtendedGamepad> = controller.extendedGamepad()?;
            let dpad = pad.dpad();
            let (left, right) = (pad.leftThumbstick(), pad.rightThumbstick());
            Some(Snapshot {
                category: controller.productCategory().to_string(),
                a: pressed(Some(pad.buttonA())),
                b: pressed(Some(pad.buttonB())),
                x: pressed(Some(pad.buttonX())),
                y: pressed(Some(pad.buttonY())),
                left_shoulder: pressed(Some(pad.leftShoulder())),
                right_shoulder: pressed(Some(pad.rightShoulder())),
                options: pressed(pad.buttonOptions()),
                menu: pressed(Some(pad.buttonMenu())),
                left_stick_button: pressed(pad.leftThumbstickButton()),
                right_stick_button: pressed(pad.rightThumbstickButton()),
                dpad_up: pressed(Some(dpad.up())),
                dpad_down: pressed(Some(dpad.down())),
                dpad_left: pressed(Some(dpad.left())),
                dpad_right: pressed(Some(dpad.right())),
                left_stick: [left.xAxis().value(), left.yAxis().value()],
                right_stick: [right.xAxis().value(), right.yAxis().value()],
                triggers: [pad.leftTrigger().value(), pad.rightTrigger().value()],
            })
        }
    }
}

/// Every controller iOS has paired.
pub fn controllers() -> Vec<Controller> {
    #[cfg(target_os = "ios")]
    return ios::controllers();
    #[cfg(not(target_os = "ios"))]
    Vec::new()
}

/// The pad that drives the game, read now: the one iOS last saw used, else the first with a full gamepad
/// profile. `None` when no such pad is connected.
pub fn active() -> Option<Snapshot> {
    #[cfg(target_os = "ios")]
    return ios::active();
    #[cfg(not(target_os = "ios"))]
    None
}

/// Whether an external keyboard is currently attached (main-thread UIKit input use only).
pub fn keyboard_connected() -> bool {
    #[cfg(target_os = "ios")]
    // SAFETY: a read-only GameController class property, on the game's main thread.
    return unsafe { objc2_game_controller::GCKeyboard::coalescedKeyboard().is_some() };
    #[cfg(not(target_os = "ios"))]
    false
}

/// Whether a hardware mouse is currently attached (main-thread UIKit input use only).
pub fn mouse_connected() -> bool {
    #[cfg(target_os = "ios")]
    // SAFETY: a read-only GameController class property, on the game's main thread.
    return unsafe { !objc2_game_controller::GCMouse::mice().is_empty() };
    #[cfg(not(target_os = "ios"))]
    false
}
