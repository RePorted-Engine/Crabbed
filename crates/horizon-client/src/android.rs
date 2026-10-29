//! Android's way in: NativeActivity loads `libhorizon_android.so` (crates/horizon-android) and calls
//! `android_main` on its own thread, where `main` then runs as it does on every other platform.
use std::sync::OnceLock;
use winit::platform::android::activity::AndroidApp;

/// The activity, for `game::event_loop`.
pub(crate) static APP: OnceLock<AndroidApp> = OnceLock::new();

#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Saves, resources and imported archives go where `adb push` reaches, as Documents does on iOS:
    // Android/data/dev.horizon.game/files. The game's home (`$HOME/.local/share/horizon`: logs, crash reports,
    // settings) goes there too, since an app has no HOME and a log nobody can pull is no use. Set before any
    // other thread of ours exists.
    if let Some(files) = app
        .external_data_path()
        .or_else(|| app.internal_data_path())
    {
        let _ = std::fs::create_dir_all(&files);
        let _ = std::env::set_current_dir(&files);
        unsafe { std::env::set_var("HOME", files) };
    }
    let _ = APP.set(app);
    crate::main();
}
