//! Horizon: native Rust client for Minecraft Java Edition 26.3 gameplay.
// Android's entry point has to be an unmangled symbol; everywhere else there is no unsafe code at all.
#![cfg_attr(not(target_os = "android"), forbid(unsafe_code))]
#[cfg(target_os = "android")]
mod android;
mod game;

use horizon_client::{crash, logging};

/// The mod API level this game serves, kept in the program for the launcher
/// to read without running it (`horizon_mods::api_level::MARK`). `main`
/// reads it, so the linker keeps it.
static MOD_API_LEVEL: [u8; 27] = horizon_mods::api_level::MARK;

/// Start the game where a phone lets it write.
///
/// iOS hands an app a working directory of `/`, which is read-only, while the game builds its world saves,
/// its `options.txt`, its `.horizon-cache` and its `resourcepacks/` from `current_dir()` -- so every one of
/// those is a path it cannot create. The writable place is the app's data container, and `$HOME` names it,
/// so the fix is one call made before anything reads a path: work in `Documents`. The launcher runs the game
/// inside its game home instead; a phone has no launcher, and `Documents` is the folder the Files app shows,
/// which is where a player drops a resource pack or a world.
///
/// Prepared resources are looked for by `Options::read`, which checks a `resources/` beside the
/// executable first and then the `resources/` that `horizon_client::import` unpacks into this very
/// folder -- so both the assets shipped in the bundle and the ones carried in by a player are found
/// with no argument to pass.
#[cfg(target_os = "ios")]
fn into_app_sandbox() -> Result<std::path::PathBuf, String> {
    let documents = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(|home| std::path::PathBuf::from(home).join("Documents"))
        .ok_or("iOS gave this app no HOME")?;
    let describe = |error: std::io::Error| format!("{}: {error}", documents.display());
    std::fs::create_dir_all(&documents).map_err(describe)?;
    std::env::set_current_dir(&documents).map_err(describe)?;
    Ok(documents)
}

fn main() {
    // Before the event loop: iOS asks for the AirPlay scene's delegate by name when a display connects.
    horizon_airplay::register();
    horizon_airplay::audio::start();
    std::hint::black_box(&MOD_API_LEVEL);
    #[cfg(target_os = "ios")]
    let documents = match into_app_sandbox() {
        Ok(documents) => documents,
        Err(error) => {
            eprintln!("Horizon: {error}");
            std::process::exit(1);
        }
    };
    // `android_main` already moved into the app's files folder.
    #[cfg(target_os = "android")]
    let documents = std::env::current_dir().unwrap_or_default();
    horizon_client::telemetry::mark_start();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let game_args = args.iter().take_while(|a| *a != "--dedicated");
    let help = game_args.clone().any(|a| a == "--help" || a == "-h");
    // The dedicated server logs in its own folder, as vanilla's does; the game in its home.
    let (home, side, thread) = if args.iter().any(|a| a == "--dedicated") {
        (std::path::PathBuf::from("."), "server", "Server thread")
    } else {
        (logging::game_home(), "client", "Render thread")
    };
    if !help {
        logging::init(&home, thread);
        // The log's first line names the build, so a bug report that carries the log says which one it was.
        log::info!("Horizon {}", crash::version());
        crash::install(home.clone(), side);
        // The Microsoft account a last start kept (the title screen's face button), signed in again.
        #[cfg(not(target_os = "ios"))]
        if side == "client" {
            horizon_client::msa::restore(&home);
        }
    }
    // What a phone does with its files before the game looks for them, once the log can say so.
    #[cfg(any(target_os = "ios", target_os = "android"))]
    if !help {
        // A swap that a flat battery or a crash left half done is finished, or the old tree put back,
        // before anything reads the folder it was working on.
        match horizon_client::import::recover(&documents) {
            Ok(Some(path)) => log::info!("Horizon: finished the import at {}", path.display()),
            Ok(None) => {}
            Err(error) => log::error!("Horizon: could not finish the last import: {error}"),
        }
        // The cable's door: an archive dragged into `Documents` is installed here, so the launch it was
        // dragged for is the one that finds it. `scan` says what it took and what it left alone.
        horizon_client::import::scan(&documents);
    }
    if let Err(error) = game::run() {
        log::error!("Horizon: {error}");
        if !help {
            crash::report_error(&home, side, &*error);
        }
        // Started from Explorer, the console window closes with the game:
        // keep the message on screen until it is read.
        #[cfg(windows)]
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            eprintln!("Press Enter to close.");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        std::process::exit(1);
    }
    logging::flush();
}
