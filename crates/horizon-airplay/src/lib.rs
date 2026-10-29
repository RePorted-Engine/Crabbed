//! The game on an AirPlay display. An app that declares the external display scene role
//! (`UIWindowSceneSessionRoleExternalDisplayNonInteractive` in GameInfo.plist) gets that display as a screen
//! of its own instead of a mirror of the phone: iOS connects a scene to [`register`]'s delegate, which puts a
//! window in it, and [`screen`] hands that window's view to the game for wgpu to draw on. The phone, meanwhile,
//! is covered by a view that says "Playing on AirPlay" and takes its touches: on AirPlay the game is played
//! with a controller.
//!
//! Splitscreen splits the display top and bottom: [`screen`] then hands the top half to the first player and
//! [`second_screen`] the bottom half to the second, whose controller is the phone. The cover then draws the
//! second player's touch controls ([`phone_controls`]) and lets the touches through to the game under it.
//!
//! The game forbids `unsafe`, so the Objective-C lives here. Elsewhere [`screen`] is always `None`.

pub mod audio;

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

/// A view on the AirPlay display, while one is connected: what a player's renderer draws to instead of the
/// phone's window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    /// Unique to the view, so a display that went and came back, or a half that was made again, is a new
    /// surface.
    pub id: u64,
    /// The view's size in pixels.
    pub size: (u32, u32),
    #[cfg(target_os = "ios")]
    view: std::ptr::NonNull<std::ffi::c_void>,
}

// SAFETY: the view is only handed to wgpu, which reads it on the main thread when it makes the surface, and
// `ios::STATE` keeps it alive while the display is connected.
#[cfg(target_os = "ios")]
unsafe impl Send for Screen {}
#[cfg(target_os = "ios")]
unsafe impl Sync for Screen {}

impl HasWindowHandle for Screen {
    #[cfg(target_os = "ios")]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = raw_window_handle::UiKitWindowHandle::new(self.view);
        // SAFETY: the view is alive for as long as this display is connected (see `Screen`).
        Ok(unsafe { WindowHandle::borrow_raw(handle.into()) })
    }
    #[cfg(not(target_os = "ios"))]
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for Screen {
    #[cfg(target_os = "ios")]
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::uikit())
    }
    #[cfg(not(target_os = "ios"))]
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

/// A touch control drawn on the phone for splitscreen's second player.
#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    pub label: String,
    /// `[x, y, width, height]` in the phone window's pixels.
    pub rect: [f32; 4],
    /// A finger holds it.
    pub held: bool,
}

/// Make the scene delegate's class known before iOS asks for it by name: call it before the event loop.
pub fn register() {
    #[cfg(target_os = "ios")]
    ios::register();
}

/// The first player's view of the AirPlay display, if one is connected: all of it, or its top half while
/// `split`. Called every frame: `cover_phone` says whether the phone shows the cover over everything (while
/// the game draws on the display) or what is under it (the launcher, which needs its touches).
pub fn screen(cover_phone: bool, split: bool) -> Option<Screen> {
    #[cfg(target_os = "ios")]
    return ios::screen(cover_phone, split);
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (cover_phone, split);
        None
    }
}

/// The second player's view: the bottom half, while [`screen`] splits the display.
pub fn second_screen() -> Option<Screen> {
    #[cfg(target_os = "ios")]
    return ios::second_screen();
    #[cfg(not(target_os = "ios"))]
    None
}

/// The phone shows the game under its cover: splitscreen's second player's menus, which they tap there.
/// Called every frame in splitscreen; off again when it ends.
pub fn phone_shows_game(on: bool) {
    #[cfg(target_os = "ios")]
    ios::phone_shows_game(on);
    #[cfg(not(target_os = "ios"))]
    let _ = on;
}

/// The second player's touch controls on the phone, which then lets its touches through to the game; `None`
/// takes them away and the cover takes the touches again.
pub fn phone_controls(buttons: Option<&[Button]>) {
    #[cfg(target_os = "ios")]
    ios::phone_controls(buttons);
    #[cfg(not(target_os = "ios"))]
    let _ = buttons;
}

#[cfg(target_os = "ios")]
mod ios {
    use super::{Button, Screen};
    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly, Message, define_class};
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
    use objc2_ui_kit::{
        NSTextAlignment, UIApplication, UIColor, UIFont, UILabel, UIScene,
        UISceneConnectionOptions, UISceneDelegate, UISceneSession, UIView, UIViewAutoresizing,
        UIViewController, UIWindow, UIWindowScene, UIWindowSceneDelegate,
        UIWindowSceneSessionRoleApplication,
    };
    use std::cell::{Cell, RefCell};
    use std::ptr::NonNull;

    /// A view with the id its surfaces know it by.
    struct View {
        id: u64,
        view: Retained<UIView>,
    }

    struct Cover {
        view: Retained<UIView>,
        label: Retained<UILabel>,
        /// The second player's controls, over the cover's black.
        pad: Retained<UIView>,
        drawn: Vec<Button>,
    }

    struct State {
        scene: Retained<UIWindowScene>,
        /// Kept: a window nothing retains leaves the display.
        _window: Retained<UIWindow>,
        view: View,
        /// The top and bottom halves, while split.
        halves: Option<(View, View)>,
        cover: Option<Cover>,
        idle: Retained<UILabel>,
    }

    thread_local! {
        static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
        static IDS: Cell<u64> = const { Cell::new(0) };
        /// `phone_shows_game`: the cover stands aside.
        static PHONE_GAME: Cell<bool> = const { Cell::new(false) };
    }

    fn next_id() -> u64 {
        IDS.with(|c| {
            c.set(c.get() + 1);
            c.get()
        })
    }

    define_class!(
        // The name GameInfo.plist's external display configuration gives.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "HorizonAirPlayDelegate"]
        struct Delegate;

        unsafe impl NSObjectProtocol for Delegate {}

        unsafe impl UISceneDelegate for Delegate {
            #[unsafe(method(scene:willConnectToSession:options:))]
            fn will_connect(
                &self,
                scene: &UIScene,
                _session: &UISceneSession,
                _options: &UISceneConnectionOptions,
            ) {
                connect(scene);
            }

            #[unsafe(method(sceneDidDisconnect:))]
            fn did_disconnect(&self, scene: &UIScene) {
                disconnect(scene);
            }
        }

        unsafe impl UIWindowSceneDelegate for Delegate {}
    );

    pub(super) fn register() {
        let _ = <Delegate as objc2::ClassType>::class();
    }

    fn flexible() -> UIViewAutoresizing {
        UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight
    }

    fn connect(scene: &UIScene) {
        let Some(scene) = scene.downcast_ref::<UIWindowScene>() else {
            return;
        };
        let mtm = MainThreadMarker::from(scene);
        let window = UIWindow::initWithWindowScene(UIWindow::alloc(mtm), scene);
        let controller = UIViewController::new(mtm);
        let view = controller.view().unwrap_or_else(|| UIView::new(mtm));
        view.setBackgroundColor(Some(&UIColor::blackColor()));
        window.setRootViewController(Some(&controller));
        window.setHidden(false);
        let idle = label(mtm, view.bounds(), "Game is not running", 28.0);
        idle.setAutoresizingMask(flexible());
        idle.setBackgroundColor(Some(&UIColor::blackColor()));
        view.addSubview(&idle);
        STATE.with(|s| {
            *s.borrow_mut() = Some(State {
                scene: scene.retain(),
                _window: window,
                view: View {
                    id: next_id(),
                    view,
                },
                halves: None,
                cover: None,
                idle,
            })
        });
    }

    fn disconnect(scene: &UIScene) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if state.as_ref().is_some_and(|st| {
                std::ptr::eq(&*st.scene as *const UIWindowScene as *const UIScene, scene)
            }) && let Some(st) = state.take()
                && let Some(cover) = st.cover
            {
                cover.view.removeFromSuperview();
            }
        });
    }

    /// The phone's own window: the one in the scene with the application role.
    fn phone_window(mtm: MainThreadMarker) -> Option<Retained<UIWindow>> {
        let app = UIApplication::sharedApplication(mtm);
        // SAFETY: a UIKit constant, read on the main thread.
        let role = unsafe { UIWindowSceneSessionRoleApplication };
        app.connectedScenes().iter().find_map(|scene| {
            let scene = scene.downcast::<UIWindowScene>().ok()?;
            if *scene.session().role() != *role {
                return None;
            }
            scene.windows().firstObject()
        })
    }

    fn label(mtm: MainThreadMarker, frame: CGRect, text: &str, size: f64) -> Retained<UILabel> {
        let label = UILabel::initWithFrame(UILabel::alloc(mtm), frame);
        label.setText(Some(&NSString::from_str(text)));
        // SAFETY: a label made here, on the main thread.
        unsafe { label.setTextColor(Some(&UIColor::whiteColor())) };
        label.setTextAlignment(NSTextAlignment::Center);
        unsafe { label.setFont(Some(&UIFont::boldSystemFontOfSize(size))) };
        label
    }

    /// A black view over the phone's window that says where the game is, and takes the touches the hidden
    /// controls would have. Made when first shown: the phone's window may not exist yet when a display that
    /// was already connected joins at launch.
    fn cover_phone(mtm: MainThreadMarker) -> Option<Cover> {
        let window = phone_window(mtm)?;
        let bounds: CGRect = window.bounds();
        let view = UIView::initWithFrame(UIView::alloc(mtm), bounds);
        view.setAutoresizingMask(flexible());
        view.setBackgroundColor(Some(&UIColor::blackColor()));
        let text = label(mtm, bounds, "Playing on AirPlay", 22.0);
        text.setAutoresizingMask(flexible());
        view.addSubview(&text);
        let pad = UIView::initWithFrame(UIView::alloc(mtm), bounds);
        pad.setAutoresizingMask(flexible());
        view.addSubview(&pad);
        window.addSubview(&view);
        Some(Cover {
            view,
            label: text,
            pad,
            drawn: Vec::new(),
        })
    }

    fn screen_of(state: &State, view: &View) -> Screen {
        let bounds = view.view.bounds();
        let scale = state.scene.screen().scale();
        Screen {
            id: view.id,
            size: (
                (bounds.size.width * scale).round().max(1.0) as u32,
                (bounds.size.height * scale).round().max(1.0) as u32,
            ),
            view: NonNull::from(&*view.view).cast(),
        }
    }

    /// The display's halves, made new each time it splits: added over the whole view's own layers, which may
    /// still hold the last frame drawn on all of it.
    fn halves(state: &State) -> (View, View) {
        let mtm = MainThreadMarker::from(&*state.view.view);
        let bounds = state.view.view.bounds();
        let half = bounds.size.height / 2.0;
        let make = |y: f64, mask: UIViewAutoresizing| {
            let frame = CGRect::new(CGPoint::new(0.0, y), CGSize::new(bounds.size.width, half));
            let view = UIView::initWithFrame(UIView::alloc(mtm), frame);
            view.setAutoresizingMask(
                mask | UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
            );
            view.setBackgroundColor(Some(&UIColor::blackColor()));
            state.view.view.addSubview(&view);
            View {
                id: next_id(),
                view,
            }
        };
        (
            make(0.0, UIViewAutoresizing::FlexibleBottomMargin),
            make(half, UIViewAutoresizing::FlexibleTopMargin),
        )
    }

    pub(super) fn screen(cover: bool, split: bool) -> Option<Screen> {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let state = state.as_mut()?;
            if cover && state.cover.is_none() {
                state.cover = cover_phone(MainThreadMarker::from(&*state.view.view));
            }
            state.idle.setHidden(cover);
            if !cover {
                state.view.view.bringSubviewToFront(&state.idle);
            }
            let playing = cover;
            let cover = cover && !PHONE_GAME.with(Cell::get);
            if let Some(c) = &state.cover {
                c.view.setHidden(!cover);
                // The launcher or the keyboard may have put a view over it since.
                if cover && let Some(window) = c.view.superview() {
                    window.bringSubviewToFront(&c.view);
                }
            }
            if split && state.halves.is_none() {
                state.halves = Some(halves(state));
            } else if !split && let Some((top, bottom)) = state.halves.take() {
                top.view.removeFromSuperview();
                bottom.view.removeFromSuperview();
            }
            let view = state.halves.as_ref().map_or(&state.view, |(top, _)| top);
            playing.then(|| screen_of(state, view))
        })
    }

    pub(super) fn second_screen() -> Option<Screen> {
        STATE.with(|s| {
            let state = s.borrow();
            let state = state.as_ref()?;
            let (_, bottom) = state.halves.as_ref()?;
            Some(screen_of(state, bottom))
        })
    }

    pub(super) fn phone_shows_game(on: bool) {
        PHONE_GAME.with(|p| p.set(on));
        STATE.with(|s| {
            if let Some(cover) = s.borrow().as_ref().and_then(|st| st.cover.as_ref())
                && on
            {
                cover.view.setHidden(true);
            }
        });
    }

    pub(super) fn phone_controls(buttons: Option<&[Button]>) {
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            let Some(cover) = state.as_mut().and_then(|st| st.cover.as_mut()) else {
                return;
            };
            // With controls, the touches go through to the game's view under the cover.
            cover.view.setUserInteractionEnabled(buttons.is_none());
            cover.label.setHidden(buttons.is_some());
            let buttons = buttons.unwrap_or_default();
            if cover.drawn == buttons {
                return;
            }
            let mtm = MainThreadMarker::from(&*cover.pad);
            for old in cover.pad.subviews().iter() {
                old.removeFromSuperview();
            }
            let scale = cover.view.contentScaleFactor().max(1.0);
            for b in buttons {
                let frame = CGRect::new(
                    CGPoint::new(f64::from(b.rect[0]) / scale, f64::from(b.rect[1]) / scale),
                    CGSize::new(f64::from(b.rect[2]) / scale, f64::from(b.rect[3]) / scale),
                );
                let view = UIView::initWithFrame(UIView::alloc(mtm), frame);
                let alpha = if b.held { 0.55 } else { 0.25 };
                view.setBackgroundColor(Some(&UIColor::colorWithWhite_alpha(1.0, alpha)));
                let size = (frame.size.height * 0.45).clamp(9.0, 24.0);
                let text = label(
                    mtm,
                    CGRect::new(CGPoint::new(0.0, 0.0), frame.size),
                    &b.label,
                    size,
                );
                text.setAutoresizingMask(flexible());
                view.addSubview(&text);
                cover.pad.addSubview(&view);
            }
            cover.drawn = buttons.to_vec();
        });
    }
}
