//! Custom images declared by `image.<uniform>` in shaders.properties.
use crate::Properties;

#[derive(Clone, Debug, PartialEq)]
pub struct ImageDefinition {
    pub name: String,
    /// The sampled texture name which aliases the image's storage.
    pub sampler: String,
    /// GLSL storage format, such as `r8ui` or `rgba16f`.
    pub format: String,
    /// Clear before each frame, otherwise preserve the previous frame's contents.
    pub clear: bool,
    pub relative: bool,
    /// One to three dimensions, in pixels or relative to the framebuffer.
    pub size: Vec<f32>,
}

impl ImageDefinition {
    pub fn parse(properties: &Properties) -> Result<Vec<Self>, Vec<String>> {
        let mut images = Vec::new();
        let mut errors = Vec::new();
        for entry in &properties.entries {
            let Some(name) = entry.key.strip_prefix("image.") else {
                continue;
            };
            let words: Vec<_> = entry.value.split_whitespace().collect();
            let parsed = (|| {
                if name.is_empty() || !(7..=9).contains(&words.len()) {
                    return None;
                }
                let boolean = |word| match word {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                };
                let size = words[6..]
                    .iter()
                    .map(|word| {
                        word.parse::<f32>()
                            .ok()
                            .filter(|v| v.is_finite() && *v > 0.0)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Self {
                    name: name.into(),
                    sampler: words[0].into(),
                    format: words[2].to_ascii_lowercase(),
                    clear: boolean(words[4])?,
                    relative: boolean(words[5])?,
                    size,
                })
            })();
            match parsed {
                Some(image) => images.push(image),
                None => errors.push(format!(
                    "shaders.properties:{}: invalid custom image {}",
                    entry.line, entry.key
                )),
            }
        }
        if errors.is_empty() {
            Ok(images)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Macros;
    #[test]
    fn image_alias_dimensions_and_persistence_are_preserved() {
        let p = Properties::parse(
            "image.voxel=voxels red_integer r8ui unsigned_int true false 192 96 192\nimage.history=historyTex rgba rgba16f half_float false true 0.5 0.5",
            &Macros::standard(),
        );
        let images = ImageDefinition::parse(&p).unwrap();
        assert_eq!(images[0].size, [192.0, 96.0, 192.0]);
        assert_eq!(images[0].sampler, "voxels");
        assert!(images[0].clear && !images[0].relative);
        assert_eq!(images[1].format, "rgba16f");
        assert!(!images[1].clear && images[1].relative);
        for size in ["0", "-2", "NaN", "inf"] {
            let p = Properties::parse(
                &format!("image.bad=t rgba rgba8 unsigned_byte true false {size}"),
                &Macros::standard(),
            );
            assert!(ImageDefinition::parse(&p).is_err());
        }
    }
}
