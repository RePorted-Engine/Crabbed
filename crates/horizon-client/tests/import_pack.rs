//! The PC half, run the way a player runs it: `horizon-client --pack-resources DIR --out FILE.zip`.
//!
//! It is the real binary rather than a call of the library function, because what is being checked is
//! the argument parsing in front of it and the fact that the command quits before it asks for a
//! window, a GPU or a prepared folder. The archive it writes is then read back by
//! `horizon_client::import` -- the phone's half -- on the same machine, which is the whole promise the
//! two ends make to each other.
//!
//! Per the repository's rule about real player data, the game runs with fresh temporary homes.
use horizon_client::import::{Kind, Pack, resources_root};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn home(tag: &str) -> PathBuf {
    // The process id and the caller's own tag: these three run as three threads of one process, and
    // each needs a folder of its own.
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "horizon-pack-cli-{}-{tag}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A prepared tree in miniature: the manifest the archive is recognised by, and files under it.
fn prepared(dir: &Path) -> PathBuf {
    let folder = dir.join("Horizon-26.3");
    fs::create_dir_all(folder.join("assets/minecraft/models")).unwrap();
    fs::write(folder.join("horizon-resources.json"), r#"{"schema":7}"#).unwrap();
    fs::write(folder.join("facts.json"), "{}").unwrap();
    fs::write(folder.join("assets/minecraft/models/block.json"), "{}").unwrap();
    fs::write(folder.join("session.lock"), "someone is inside").unwrap();
    folder
}

#[test]
fn the_command_writes_an_archive_the_phone_installs() {
    let dir = home("out");
    let folder = prepared(&dir);
    let out = dir.join("to-phone.zip");
    let run = Command::new(env!("CARGO_BIN_EXE_horizon-client"))
        .args(["--pack-resources", folder.to_str().unwrap()])
        .args(["--out", out.to_str().unwrap()])
        .env("HORIZON_HOME", &dir)
        .env("XDG_DATA_HOME", &dir)
        .env("XDG_CONFIG_HOME", &dir)
        .env("HOME", &dir)
        .env("HORIZON_NO_AUDIO", "1")
        .output()
        .expect("the game binary runs");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "{}: {stdout}{}",
        run.status,
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(stdout.contains("packed game files"), "{stdout}");
    assert!(out.is_file(), "the archive is where --out said");
    // Three files: the running world's session.lock is not part of what a player means to send.
    let documents = dir.join("Documents");
    fs::create_dir_all(&documents).unwrap();
    let installed = Pack::open(&out).unwrap().install(&documents).unwrap();
    assert_eq!(installed.kind, Kind::Resources);
    assert_eq!(installed.files, 3);
    assert_eq!(
        resources_root(&dir.join("Horizon.app/resources"), Some(&documents)),
        Some(documents.join("resources"))
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn without_out_the_archive_lands_beside_the_folder_it_came_from() {
    let dir = home("default");
    let folder = prepared(&dir);
    let run = Command::new(env!("CARGO_BIN_EXE_horizon-client"))
        .args(["--pack-resources", folder.to_str().unwrap()])
        .env("HORIZON_HOME", &dir)
        .env("XDG_DATA_HOME", &dir)
        .env("XDG_CONFIG_HOME", &dir)
        .env("HOME", &dir)
        .env("HORIZON_NO_AUDIO", "1")
        .output()
        .expect("the game binary runs");
    assert!(run.status.success(), "{}", run.status);
    // The folder is `Horizon-26.3`, whose own extension is `3`: the name is the whole file name, so
    // the archive a player finds is `Horizon-26.3.zip` and not something they did not ask for.
    assert!(
        dir.join("Horizon-26.3.zip").is_file(),
        "wrote to {:?}",
        folder.parent().unwrap()
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_folder_that_is_not_one_of_the_three_is_refused() {
    let dir = home("refused");
    fs::create_dir_all(dir.join("holiday")).unwrap();
    fs::write(dir.join("holiday/beach.jpg"), "x").unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_horizon-client"))
        .args(["--pack-resources", dir.join("holiday").to_str().unwrap()])
        .args(["--out", dir.join("out.zip").to_str().unwrap()])
        .env("HORIZON_HOME", &dir)
        .env("XDG_DATA_HOME", &dir)
        .env("XDG_CONFIG_HOME", &dir)
        .env("HOME", &dir)
        .env("HORIZON_NO_AUDIO", "1")
        .output()
        .expect("the game binary runs");
    assert!(!run.status.success(), "packing a folder of photos");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("none of the three kinds"),
        "the failure says what is wrong: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    // Nothing half-written is left behind to be sent by mistake.
    assert!(!dir.join("out.zip").exists());
    fs::remove_dir_all(dir).unwrap();
}
