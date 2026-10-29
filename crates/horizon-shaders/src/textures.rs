//! Pack-owned image textures selected by shaders.properties. Decoding belongs to the renderer.
use crate::{Pack, Properties};

#[derive(Clone, Debug, PartialEq)]
pub struct TextureDefinition {
    /// GLSL sampler name. `texture.noise` supplies `noisetex`.
    pub name: String,
    /// Program family (`gbuffers`, `deferred`, ...), or every program for customTexture/noise.
    pub stage: Option<String>,
    /// Normalized path inside the pack's shaders folder.
    pub path: String,
    pub bytes: Vec<u8>,
    /// Optional Minecraft texture metadata, including blur/clamp sampling settings.
    pub metadata: Option<String>,
}

impl TextureDefinition {
    /// Loads only active property entries. Bytes remain in memory and no pack assets are exported.
    pub fn load(pack: &Pack, properties: &Properties) -> Result<Vec<Self>, Vec<String>> {
        let mut textures = Vec::new();
        let mut errors = Vec::new();
        for entry in &properties.entries {
            let binding = if let Some(name) = entry.key.strip_prefix("customTexture.") {
                Some((None, name))
            } else if entry.key == "texture.noise" {
                Some((None, "noisetex"))
            } else if let Some(rest) = entry.key.strip_prefix("texture.") {
                rest.split_once('.')
                    .map(|(stage, name)| (Some(stage), name))
            } else {
                continue;
            };
            let fail = |reason: &str| {
                format!("shaders.properties:{}: {}: {reason}", entry.line, entry.key)
            };
            let Some((stage, name)) = binding else {
                errors.push(fail("invalid texture binding"));
                continue;
            };
            let identifier = |s: &str| {
                !s.is_empty()
                    && s.bytes().enumerate().all(|(i, b)| {
                        b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit())
                    })
            };
            if !identifier(name) || stage.is_some_and(|s| !identifier(s)) {
                errors.push(fail("invalid program or sampler name"));
                continue;
            }
            let words: Vec<_> = entry.value.split_whitespace().collect();
            if words.len() != 1 {
                errors.push(fail(
                    "expected one image path; raw texture declarations are not supported",
                ));
                continue;
            }
            let Some(path) = crate::pack::normalize(words[0]).filter(|p| !p.is_empty()) else {
                errors.push(fail("texture path leaves the shaders folder"));
                continue;
            };
            let Some(bytes) = pack.read(&path) else {
                errors.push(fail(&format!("missing texture {path}")));
                continue;
            };
            let metadata = pack.read_text(&format!("{path}.mcmeta"));
            textures.push(Self {
                name: name.into(),
                stage: stage.map(str::to_owned),
                path,
                bytes,
                metadata,
            });
        }
        if errors.is_empty() {
            Ok(textures)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Macros, pack::tests::zip};

    #[test]
    fn aliases_stage_overrides_paths_and_metadata_survive_loading() {
        let pack = Pack::from_zip_bytes(
            "T.zip",
            zip(&[
                ("shaders/tex/a.png", "image bytes"),
                (
                    "shaders/tex/a.png.mcmeta",
                    "{\"texture\":{\"blur\":true,\"clamp\":false}}",
                ),
            ]),
        )
        .unwrap();
        let properties = Properties::parse(
            "customTexture.milkyWayTex=/tex/a.png\ntexture.noise=tex/a.png\ntexture.deferred.colortex3=tex/./a.png\ntexture.gbuffers.gaux4=tex/a.png",
            &Macros::standard(),
        );
        let textures = TextureDefinition::load(&pack, &properties).unwrap();
        assert_eq!(textures.len(), 4);
        assert_eq!(textures[0].name, "milkyWayTex");
        assert_eq!(textures[0].stage, None);
        assert_eq!(textures[0].path, "tex/a.png");
        assert_eq!(textures[0].bytes, b"image bytes");
        assert!(textures[0].metadata.as_ref().unwrap().contains("blur"));
        assert_eq!(textures[1].name, "noisetex");
        assert_eq!(textures[2].stage.as_deref(), Some("deferred"));
        assert_eq!(textures[3].name, "gaux4");
    }

    #[test]
    fn dimension_option_conditions_select_only_the_active_file() {
        let pack=Pack::from_zip_bytes("D.zip",zip(&[
            ("shaders/world0/final.fsh","#version 450\n#define OVERWORLD\nvoid main(){}"),
            ("shaders/world1/final.fsh","#version 450\n#define END\nvoid main(){}"),
            ("shaders/shaders.properties","#ifdef OVERWORLD\ncustomTexture.sky=tex/day.png\n#endif\n#ifdef END\ncustomTexture.sky=tex/end.png\n#endif"),
            ("shaders/tex/day.png","day"),
            ("shaders/tex/end.png","end"),
        ])).unwrap();
        for (folder, expected) in [("world0", "tex/day.png"), ("world1", "tex/end.png")] {
            let properties = Properties::from_pack(&pack, folder, &Macros::standard());
            let textures = TextureDefinition::load(&pack, &properties).unwrap();
            assert_eq!(textures.len(), 1);
            assert_eq!(textures[0].path, expected);
        }
    }

    #[test]
    fn missing_escaping_and_invalid_bindings_are_errors() {
        let pack = Pack::from_zip_bytes("Bad.zip", zip(&[("shaders/ok.png", "ok")])).unwrap();
        for property in [
            "customTexture.sky=../outside.png",
            "customTexture.sky=missing.png",
            "texture..sky=ok.png",
            "texture.deferred.=ok.png",
            "customTexture.sky=ok.png TEXTURE_3D",
        ] {
            let p = Properties::parse(property, &Macros::standard());
            assert!(TextureDefinition::load(&pack, &p).is_err(), "{property}");
        }
        let p = Properties::parse(
            "#if 0\ncustomTexture.sky=missing.png\n#endif",
            &Macros::standard(),
        );
        assert!(TextureDefinition::load(&pack, &p).unwrap().is_empty());
    }
}
