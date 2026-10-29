//! Relative mouse input from Apple's public GameController API (iPadOS 14+).
//! Handlers run on GCDevice's default main queue, the same thread as UIKit/winit.
use super::{
    app_state::{self, EventWrapper},
    DEVICE_ID,
};
use crate::{
    event::{
        DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    window::WindowId,
};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send, sel};
use objc2_foundation::{MainThreadMarker, NSArray};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[link(name = "GameController", kind = "framework")]
extern "C" {}

#[derive(Default)]
pub(super) struct Mouse {
    captured: Rc<Cell<bool>>,
    mice: RefCell<Vec<Retained<AnyObject>>>,
}

fn device(event: DeviceEvent) {
    app_state::handle_nonuser_event(MainThreadMarker::new().unwrap(),
        EventWrapper::StaticEvent(Event::DeviceEvent { device_id: DEVICE_ID, event }));
}

fn send(window_id: WindowId, event: WindowEvent) {
    app_state::handle_nonuser_event(
        MainThreadMarker::new().unwrap(),
        EventWrapper::StaticEvent(Event::WindowEvent { window_id, event }),
    );
}

impl Mouse {
    pub(super) fn captured(&self) -> bool { self.captured.get() }
    pub(super) fn capture(&self, captured: bool) {
        self.captured.set(captured);
    }

    pub(super) fn update(&self, window: WindowId) {
        // SAFETY: documented GCMouse/GCDevice/GCController element properties. The framework copies blocks.
        // This function and the default GCDevice handler queue are main-thread only; captured is Rc<Cell>.
        unsafe {
            let mice: Retained<NSArray<AnyObject>> = objc2::msg_send_id![class!(GCMouse), mice];
            let mut known = self.mice.borrow_mut();
            if known
                .iter()
                .any(|old| !mice.iter().any(|m| std::ptr::eq(&*m, &**old)))
            {
                // A disconnected device may never deliver button-up.
                for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                    send(
                        window,
                        WindowEvent::MouseInput {
                            device_id: DEVICE_ID,
                            state: ElementState::Released,
                            button,
                        },
                    );
                }
            }
            for mouse in mice.iter() {
                if known.iter().any(|old| std::ptr::eq(&**old, &*mouse)) {
                    continue;
                }
                let input: Retained<AnyObject> = objc2::msg_send_id![&*mouse, mouseInput];
                let moved = RcBlock::new(move |_: &AnyObject, dx: f32, dy: f32| {
                    device(DeviceEvent::MouseMotion { delta: (dx as f64, -dy as f64) });
                });
                let _: () = msg_send![&*input, setMouseMovedHandler: &*moved];
                for (selector, button) in [
                    (sel!(leftButton), MouseButton::Left),
                    (sel!(rightButton), MouseButton::Right),
                    (sel!(middleButton), MouseButton::Middle),
                ] {
                    let element: Option<Retained<AnyObject>> =
                        objc2::msg_send_id![&*input, performSelector: selector];
                    if let Some(element) = element {
                        let captured = self.captured.clone();
                        let changed = RcBlock::new(
                            move |_: &AnyObject, _: f32, down: objc2::runtime::Bool| {
                                device(DeviceEvent::Button { button: match button { MouseButton::Left => 0, MouseButton::Right => 1, _ => 2 },
                                    state: if down.as_bool() { ElementState::Pressed } else { ElementState::Released } });
                                if captured.get() {
                                    send(
                                        window,
                                        WindowEvent::MouseInput {
                                            device_id: DEVICE_ID,
                                            state: if down.as_bool() {
                                                ElementState::Pressed
                                            } else {
                                                ElementState::Released
                                            },
                                            button,
                                        },
                                    );
                                }
                            },
                        );
                        let _: () = msg_send![&*element, setPressedChangedHandler: &*changed];
                    }
                }
                let scroll: Retained<AnyObject> = objc2::msg_send_id![&*input, scroll];
                let scrolled = RcBlock::new(move |_: &AnyObject, x: f32, y: f32| {
                    device(DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(x, y) });
                    send(window, WindowEvent::MouseWheel { device_id: DEVICE_ID,
                        delta: MouseScrollDelta::LineDelta(x, y), phase: TouchPhase::Moved });
                });
                let _: () = msg_send![&*scroll, setValueChangedHandler: &*scrolled];
            }
            *known = mice
                .iter()
                .map(|m| Retained::retain(m as *const AnyObject as *mut AnyObject).unwrap())
                .collect();
        }
    }
}
