//! The game on an AirPlay display (iPhone and iPad; Horizon's own): while one is connected, the renderer draws
//! on it at its own size, the touch controls are put away, and the phone says "Playing on AirPlay" over
//! everything (crate `horizon-airplay`). It is played with a controller meanwhile. In splitscreen the display's
//! top half is this game's and its bottom half the second player's (game/splitscreen.rs). Both the game and
//! the loading screen before it call [`sync`] every frame; everywhere but iOS it finds no display and does
//! nothing.
use super::Game;
use horizon_airplay::Screen;
use horizon_client::gfx::Renderer;
use std::sync::Arc;
use winit::window::Window;

/// The view the renderer draws on: its id and size, or `None` for the phone's own window.
pub(super) type Shown = Option<(u64, (u32, u32))>;

/// Move the renderer to `screen`, a view of the AirPlay display, or back to `window` without one, when that
/// changed since `shown`.
pub(super) fn sync(
    renderer: Option<&mut Renderer>,
    window: Option<&Arc<Window>>,
    shown: &mut Shown,
    screen: Option<Screen>,
) {
    let Some(renderer) = renderer else {
        return;
    };
    let want = screen.as_ref().map(|s| (s.id, s.size));
    if want == *shown {
        return;
    }
    let phone = window.map_or((renderer.config.width, renderer.config.height), |w| {
        let size = w.inner_size();
        (size.width, size.height)
    });
    let result = match screen {
        Some(screen) => {
            let size = screen.size;
            log::info!("Horizon: AirPlay display at {}x{}", size.0, size.1);
            renderer.set_target(Some((screen.id, Arc::new(screen))), size)
        }
        None => {
            log::info!("Horizon: drawing on the phone");
            renderer.set_target(None, phone)
        }
    };
    // Not again every frame: the display is retried when it changes.
    *shown = want;
    if let Err(error) = result {
        log::warn!("Horizon: AirPlay display: {error}");
        if let Err(error) = renderer.set_target(None, phone) {
            log::error!("Horizon: back on the phone after AirPlay: {error}");
        }
    }
}

impl Game<'_> {
    /// The game draws on an AirPlay display, so the phone's window is not where it is played.
    pub(super) fn on_airplay(&self) -> bool {
        self.renderer.as_ref().is_some_and(|r| r.targeted())
    }

    pub(super) fn sync_airplay(&mut self) {
        if self.splitscreen.guest {
            return;
        }
        let screen =
            horizon_airplay::screen(!horizon_ios_launcher::covered(), self.splitscreen.active);
        sync(
            self.renderer.as_mut(),
            self.window.as_ref(),
            &mut self.airplay,
            screen,
        );
    }
}
