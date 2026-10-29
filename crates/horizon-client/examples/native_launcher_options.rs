//! Emit native launcher's option metadata without loading game files or touching player preferences.
fn main() {
    let file = horizon_client::options_file::OptionsFile::default();
    let mut options = serde_json::Map::new();
    for key in [
        "graphicsPreset",
        "renderDistance",
        "simulationDistance",
        "maxFps",
        "fov",
        "gamma",
        "guiScale",
        "particles",
        "renderClouds",
        "entityShadows",
        "bobView",
        "soundCategory_master",
        "soundCategory_music",
        "soundCategory_ambient",
        "showSubtitles",
        "mouseSensitivity",
        "autoJump",
        "invertYMouse",
    ] {
        options.insert(
            key.to_owned(),
            file.describe(key).expect("launcher option exists"),
        );
    }
    println!("{}", serde_json::json!({ "options": options }));
}
