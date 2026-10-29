//! Single-window UIKit scene lifecycle, required by iOS/iPadOS 27 for SDK 27 applications.
//! Enabled only by an app's UIApplicationSceneManifest; legacy clients retain their notification path.
use super::{
    app_state::{self, EventWrapper},
    window::WinitUIWindow,
};
use crate::{
    event::{Event, WindowEvent},
    window::WindowId,
};
use objc2::rc::Retained;
use objc2::{declare_class, mutability, ClassType, DeclaredClass};
use objc2_foundation::{ns_string, MainThreadMarker, NSBundle, NSObject, NSObjectProtocol};
use objc2_ui_kit::{
    UISceneConnectionOptions, UISceneDelegate, UISceneSession, UIWindowScene, UIWindowSceneDelegate,
};
use std::cell::{Cell, RefCell};

thread_local! {
    static SCENE: RefCell<Option<Retained<UIWindowScene>>> = const { RefCell::new(None) };
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn enabled() -> bool {
    unsafe {
        NSBundle::mainBundle()
            .objectForInfoDictionaryKey(ns_string!("UIApplicationSceneManifest"))
            .is_some()
    }
}

pub(super) fn register() {
    let _ = WinitSceneDelegate::class();
}

pub(super) fn current() -> Option<Retained<UIWindowScene>> {
    SCENE.with(|s| s.borrow().clone())
}

fn windows(scene: &UIWindowScene, focused: Option<bool>, occluded: Option<bool>) {
    let events: Vec<_> = unsafe { scene.windows() }
        .iter()
        .filter_map(|window| {
            if !window.isKindOfClass(WinitUIWindow::class()) {
                return None;
            }
            let id = WindowId(super::window::WindowId::from(
                window as *const _ as usize as u64,
            ));
            let event = match (focused, occluded) {
                (Some(value), _) => WindowEvent::Focused(value),
                (_, Some(value)) => WindowEvent::Occluded(value),
                _ => return None,
            };
            Some(EventWrapper::StaticEvent(Event::WindowEvent {
                window_id: id,
                event,
            }))
        })
        .collect();
    app_state::handle_nonuser_events(MainThreadMarker::new().unwrap(), events);
}

declare_class!(
    pub(crate) struct WinitSceneDelegate;
    unsafe impl ClassType for WinitSceneDelegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "WinitSceneDelegate";
    }
    impl DeclaredClass for WinitSceneDelegate {}
    unsafe impl NSObjectProtocol for WinitSceneDelegate {}
    unsafe impl UISceneDelegate for WinitSceneDelegate {}
    unsafe impl UIWindowSceneDelegate for WinitSceneDelegate {}

    unsafe impl WinitSceneDelegate {
        #[method(scene:willConnectToSession:options:)]
        fn connect(&self, scene: &UIWindowScene, _session: &UISceneSession, _options: &UISceneConnectionOptions) {
            // The manifest accepts the application UIWindowScene role, with multiple scenes disabled.
            SCENE.with(|s| *s.borrow_mut() = Some(unsafe { Retained::retain(scene as *const _ as *mut UIWindowScene) }.unwrap()));
            if !STARTED.with(|started| started.replace(true)) {
                app_state::did_finish_launching(MainThreadMarker::new().unwrap());
            }
        }

        #[method(sceneDidBecomeActive:)]
        fn active(&self, scene: &UIWindowScene) {
            app_state::handle_nonuser_event(MainThreadMarker::new().unwrap(), EventWrapper::StaticEvent(Event::Resumed));
            windows(scene, Some(true), None);
        }

        #[method(sceneWillResignActive:)]
        fn inactive(&self, scene: &UIWindowScene) {
            windows(scene, Some(false), None);
            app_state::handle_nonuser_event(MainThreadMarker::new().unwrap(), EventWrapper::StaticEvent(Event::Suspended));
        }

        #[method(sceneWillEnterForeground:)]
        fn foreground(&self, scene: &UIWindowScene) { windows(scene, None, Some(false)); }

        #[method(sceneDidEnterBackground:)]
        fn background(&self, scene: &UIWindowScene) { windows(scene, None, Some(true)); }

        #[method(sceneDidDisconnect:)]
        fn disconnect(&self, scene: &UIWindowScene) {
            windows(scene, Some(false), None);
            SCENE.with(|s| { s.borrow_mut().take(); });
        }
    }
);
