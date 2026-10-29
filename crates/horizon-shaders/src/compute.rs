//! Compute pass discovery and documented dispatch directives, independent of the renderer.
//! Compute stages execute in suffix order before their pass's vertex/fragment stages.

use crate::{ImageDefinition, Macros, Pack, Preprocessed, Properties, eval_condition, preprocess};

/// Where a compute stage runs in the pack's frame sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeGroup {
    /// Runs once after loading the pack or resizing its buffers.
    Setup,
    Begin,
    ShadowComp,
    Prepare,
    Deferred,
    Composite,
    Final,
}

/// The shader's documented work group count directive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorkGroups {
    Fixed([u32; 3]),
    /// Multipliers of render width and height, with one group in Z.
    Render([f32; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeDispatch {
    pub local_size: [u32; 3],
    pub work_groups: WorkGroups,
}

impl ComputeDispatch {
    /// Reads live, macro-expanded GLSL. Missing local axes default to one; missing work group
    /// directives cover the framebuffer. Invalid directives fail instead of changing the dispatch.
    pub fn parse(source: &Preprocessed) -> Result<Self, String> {
        let tokens: Vec<_> = crate::macros::lex(&source.code)
            .into_iter()
            .filter(|t| !t.trim().is_empty())
            .collect();
        let mut local_size = [1; 3];
        let mut found_local = false;
        for (axis, key) in ["local_size_x", "local_size_y", "local_size_z"]
            .iter()
            .enumerate()
        {
            let positions: Vec<_> = tokens
                .iter()
                .enumerate()
                .filter(|(_, t)| *t == key)
                .map(|(i, _)| i)
                .collect();
            if positions.len() > 1 {
                return Err(format!("duplicate {key}"));
            }
            if let Some(&i) = positions.first() {
                found_local = true;
                if tokens.get(i + 1) != Some(&"=") {
                    return Err(format!("invalid {key}"));
                }
                let end = (i + 2..tokens.len())
                    .find(|&j| matches!(tokens[j], "," | ")"))
                    .ok_or_else(|| format!("invalid {key}"))?;
                local_size[axis] = integer(&tokens[i + 2..end], false)?;
            }
        }
        if !found_local {
            return Err("missing compute local size declaration".into());
        }
        let fixed = directive(&tokens, "ivec3", "workGroups")?;
        let render = directive(&tokens, "vec2", "workGroupsRender")?;
        if fixed.is_some() && render.is_some() {
            return Err("both workGroups and workGroupsRender are declared".into());
        }
        let work_groups = if let Some(args) = fixed {
            let args = components(args, 3)?;
            WorkGroups::Fixed([
                integer(args[0], true)?,
                integer(args[1], true)?,
                integer(args[2], true)?,
            ])
        } else if let Some(args) = render {
            let args = components(args, 2)?;
            let number = |tokens: &[&str]| {
                let text = tokens.join("");
                text.trim_end_matches(['f', 'F'])
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .ok_or_else(|| format!("invalid workGroupsRender component: {text}"))
            };
            WorkGroups::Render([number(args[0])?, number(args[1])?])
        } else {
            WorkGroups::Render([1.0; 2])
        };
        Ok(Self {
            local_size,
            work_groups,
        })
    }

    /// Number of work groups, checked for representable counts. Device-specific dispatch and
    /// local-size limits are checked by the renderer before allocating or submitting a pass.
    pub fn groups(self, render_size: [u32; 2]) -> Option<[u32; 3]> {
        match self.work_groups {
            WorkGroups::Fixed(groups) => Some(groups),
            WorkGroups::Render(scale) => {
                let mut groups = [1; 3];
                for axis in 0..2 {
                    // Multiply at the declared GLSL float precision before rounding.
                    // Promoting 0.1f to f64 first would turn 10 * 0.1f into >1 and
                    // incorrectly dispatch an extra group on exact pixel boundaries.
                    let pixels = f64::from(render_size[axis] as f32 * scale[axis]).ceil();
                    let count = (pixels / f64::from(self.local_size[axis])).ceil();
                    if !count.is_finite() || count < 0.0 || count > f64::from(u32::MAX) {
                        return None;
                    }
                    groups[axis] = count as u32;
                }
                Some(groups)
            }
        }
    }
}

fn integer(tokens: &[&str], zero: bool) -> Result<u32, String> {
    let text = tokens.join("");
    let value = eval_condition(&text, &Macros::default())
        .ok()
        .and_then(|v| u32::try_from(v).ok())
        .filter(|v| zero || *v != 0);
    value.ok_or_else(|| format!("invalid compute integer: {text}"))
}

fn directive<'a>(
    tokens: &'a [&'a str],
    ty: &str,
    name: &str,
) -> Result<Option<&'a [&'a str]>, String> {
    let positions: Vec<_> = tokens
        .windows(3)
        .enumerate()
        .filter(|(_, t)| t[0] == "const" && t[1] == ty && t[2] == name)
        .map(|(i, _)| i)
        .collect();
    if positions.len() > 1 {
        return Err(format!("duplicate {name}"));
    }
    let Some(&i) = positions.first() else {
        return Ok(None);
    };
    if tokens.get(i + 3) != Some(&"=")
        || tokens.get(i + 4) != Some(&ty)
        || tokens.get(i + 5) != Some(&"(")
    {
        return Err(format!("invalid {name} constructor"));
    }
    let mut depth = 1;
    for j in i + 6..tokens.len() {
        match tokens[j] {
            "(" => depth += 1,
            ")" => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            if tokens.get(j + 1) != Some(&";") {
                return Err(format!("invalid {name} declaration"));
            }
            return Ok(Some(&tokens[i + 6..j]));
        }
    }
    Err(format!("unterminated {name} constructor"))
}

fn components<'a>(tokens: &'a [&'a str], size: usize) -> Result<Vec<&'a [&'a str]>, String> {
    let mut args = Vec::new();
    let (mut start, mut depth) = (0, 0);
    for (i, token) in tokens.iter().enumerate() {
        match *token {
            "(" => depth += 1,
            ")" => depth -= 1,
            "," if depth == 0 => {
                args.push(&tokens[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    args.push(&tokens[start..]);
    if args.len() == 1 {
        args.resize(size, args[0]);
    }
    if args.len() != size || args.iter().any(|a| a.is_empty()) {
        return Err("invalid compute vector component count".into());
    }
    Ok(args)
}

pub struct LoadedCompute {
    /// The pass name, without the optional compute suffix (e.g. `composite2`).
    pub base: String,
    /// Full name (e.g. `composite2_a`), used for diagnostics.
    pub name: String,
    pub group: ComputeGroup,
    pub program: crate::translate::ComputeProgram,
    pub dispatch: ComputeDispatch,
}

pub struct Compute {
    pub programs: Vec<LoadedCompute>,
    pub complete: bool,
    pub notes: Vec<String>,
}

/// Loads every compute stage in execution order. Dimension selection follows `load_post`:
/// `folder` replaces the base shader folder. A malformed present program makes the result
/// incomplete and carries its diagnostics; absent and explicitly disabled programs do not.
pub fn load_compute(
    pack: &Pack,
    folder: &str,
    properties: &Properties,
    macros: &Macros,
    images: &[ImageDefinition],
) -> Compute {
    let mut out = Compute {
        programs: Vec::new(),
        complete: true,
        notes: Vec::new(),
    };
    let switches = crate::switches_on(pack);
    for (group, prefix) in [
        (ComputeGroup::Setup, "setup"),
        (ComputeGroup::Begin, "begin"),
        (ComputeGroup::ShadowComp, "shadowcomp"),
        (ComputeGroup::Prepare, "prepare"),
        (ComputeGroup::Deferred, "deferred"),
        (ComputeGroup::Composite, "composite"),
        (ComputeGroup::Final, "final"),
    ] {
        for index in 0..=if group == ComputeGroup::Final { 0 } else { 99 } {
            let base = if index == 0 {
                prefix.to_owned()
            } else {
                format!("{prefix}{index}")
            };
            if !properties.program_enabled(folder, &base, macros, &switches) {
                continue;
            }
            for suffix in std::iter::once(String::new()).chain(('a'..='z').map(|c| format!("_{c}")))
            {
                let name = format!("{base}{suffix}");
                let path = if folder.is_empty() {
                    format!("{name}.csh")
                } else {
                    format!("{folder}/{name}.csh")
                };
                if !pack.exists(&path) {
                    continue;
                }
                let pp = preprocess(pack, &path, macros);
                if !pp.diagnostics.is_empty() {
                    out.complete = false;
                    out.notes.extend(pp.diagnostics.iter().map(|d| {
                        format!(
                            "{}:{}: {}",
                            pp.files
                                .get(d.origin.file as usize)
                                .map_or("?", String::as_str),
                            d.origin.line,
                            d.message
                        )
                    }));
                    continue;
                }
                let dispatch = match ComputeDispatch::parse(&pp) {
                    Ok(dispatch) => dispatch,
                    Err(error) => {
                        out.complete = false;
                        out.notes.push(format!("{path}: {error}"));
                        continue;
                    }
                };
                // Indirect dispatch requires a storage buffer; don't silently use a direct count.
                if properties.get(&format!("indirect.{name}")).is_some() {
                    out.complete = false;
                    out.notes.push(format!(
                        "{path}: indirect compute dispatch is not supported"
                    ));
                    continue;
                }
                match crate::translate::translate_compute(&pp, images) {
                    Ok(program) => out.programs.push(LoadedCompute {
                        base: base.clone(),
                        name,
                        group,
                        program,
                        dispatch,
                    }),
                    Err(errors) => {
                        out.complete = false;
                        out.notes.extend(
                            errors
                                .iter()
                                .map(|e| format!("{}:{}: {}", e.file, e.line, e.message)),
                        );
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(code: &str) -> Result<ComputeDispatch, String> {
        ComputeDispatch::parse(&Preprocessed {
            code: code.into(),
            ..Default::default()
        })
    }
    #[test]
    fn dispatch_covers_non_multiple_render_dimensions() {
        let d = parse("layout(local_size_x=16,local_size_y=8) in;").unwrap();
        assert_eq!(d.local_size, [16, 8, 1]);
        assert_eq!(d.groups([1921, 1081]), Some([121, 136, 1]));
        let d = parse("layout(local_size_x=19,local_size_z=5) in; const vec2 workGroupsRender=vec2(1.0f, 0.732f);").unwrap();
        assert_eq!(d.groups([1920, 1080]), Some([102, 791, 1]));
        let d = parse("layout(local_size_x=1) in; const vec2 workGroupsRender=vec2(0.1);").unwrap();
        assert_eq!(d.groups([10, 10]), Some([1, 1, 1]));
    }
    #[test]
    fn solas_voxel_dispatch_is_three_dimensional() {
        let d = parse("layout(local_size_x=8,local_size_y=8,local_size_z=8) in; const ivec3 workGroups=ivec3(16,8,16);").unwrap();
        assert_eq!(d.groups([1920, 1080]), Some([16, 8, 16]));
        assert_eq!(
            parse("layout(local_size_x=4) in; const ivec3 workGroups=ivec3(128/8);")
                .unwrap()
                .groups([1, 1]),
            Some([16; 3])
        );
    }
    #[test]
    fn discovery_preserves_suffix_order_dimensions_and_disabled_passes() {
        let code = "#version 430\nlayout(local_size_x=8) in;void main(){}";
        let pack = Pack::from_zip_bytes(
            "compute",
            crate::pack::tests::zip(&[
                ("shaders/shadowcomp_b.csh", code),
                ("shaders/shadowcomp.csh", code),
                ("shaders/shadowcomp_a.csh", code),
                ("shaders/shadowcomp1.csh", code),
                ("shaders/world-1/composite.csh", code),
            ]),
        )
        .unwrap();
        let macros = Macros::standard();
        let properties = Properties::parse("program.shadowcomp1.enabled=false", &macros);
        let result = load_compute(&pack, "", &properties, &macros, &[]);
        assert!(result.complete, "{:?}", result.notes);
        assert_eq!(
            result
                .programs
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["shadowcomp", "shadowcomp_a", "shadowcomp_b"]
        );
        assert!(
            result
                .programs
                .iter()
                .all(|p| p.base == "shadowcomp" && p.group == ComputeGroup::ShadowComp)
        );
        let nether = load_compute(&pack, "world-1", &properties, &macros, &[]);
        assert!(nether.complete, "{:?}", nether.notes);
        assert_eq!(nether.programs.len(), 1);
        assert_eq!(nether.programs[0].name, "composite");
    }
    #[test]
    fn missing_includes_and_indirect_dispatch_report_incomplete() {
        let code = "#version 430\nlayout(local_size_x=8) in;void main(){}";
        let pack = Pack::from_zip_bytes(
            "compute",
            crate::pack::tests::zip(&[
                ("shaders/composite.csh", "#include \"missing.glsl\""),
                ("shaders/composite_a.csh", code),
            ]),
        )
        .unwrap();
        let macros = Macros::standard();
        let properties = Properties::parse("indirect.composite_a=0 0", &macros);
        let result = load_compute(&pack, "", &properties, &macros, &[]);
        assert!(!result.complete);
        assert!(result.programs.is_empty());
        assert!(result.notes.iter().any(|n| n.contains("missing.glsl")));
        assert!(result.notes.iter().any(|n| n.contains("indirect compute")));
    }
    #[test]
    #[ignore = "needs HORIZON_TEST_COMPUTE_PACK, a local pack never committed"]
    fn real_compute_pack() {
        let path =
            std::env::var_os("HORIZON_TEST_COMPUTE_PACK").expect("HORIZON_TEST_COMPUTE_PACK");
        let pack = Pack::open(std::path::Path::new(&path)).unwrap();
        let mut macros = Macros::standard();
        macros.undef("MC_OS_MAC");
        macros.define("MC_OS_OTHER", "");
        let folder = crate::dimension_folder(&pack, 0);
        let properties = Properties::from_pack(&pack, &folder, &macros);
        let images = ImageDefinition::parse(&properties).unwrap();
        let result = load_compute(&pack, &folder, &properties, &macros, &images);
        for p in &result.programs {
            println!(
                "{}: {:?}, {:?}, images={:?}",
                p.name, p.dispatch.local_size, p.dispatch.work_groups, p.program.interface.images
            );
        }
        assert!(result.complete, "{:?}", result.notes);
        let geometry = crate::GbuffersInputs {
            declarations: "layout(location=0) in vec3 hz_v_pos;".into(),
            body: "hz_Vertex=vec4(hz_v_pos,1.0);hz_Color=vec4(1.0);".into(),
            attributes: vec![("mc_Entity".into(), "vec4(-1.0)".into())],
            ..Default::default()
        };
        let inputs = crate::GameInputs {
            terrain: Some(geometry.clone()),
            sky: Some(geometry.clone()),
            celestial: Some(geometry.clone()),
            entities: Some(geometry.clone()),
            clouds: Some(geometry.clone()),
            weather: Some(geometry.clone()),
            hand: Some(geometry),
        };
        let post = crate::load_post(&pack, &folder, &properties, &macros, &inputs);
        println!("post complete={}, notes={:?}", post.complete, post.notes);
        assert!(post.complete, "{:?}", post.notes);
    }
    #[test]
    fn invalid_dispatch_never_falls_back_to_screen_size() {
        for source in [
            "void main(){}",
            "layout(local_size_x=0) in;",
            "layout(local_size_x=8) in;const ivec3 workGroups=ivec3(-1,2,3);",
            "layout(local_size_x=8) in;const vec2 workGroupsRender=vec2(NaN);",
            "layout(local_size_x=8) in;const ivec3 workGroups=ivec3(1);const vec2 workGroupsRender=vec2(1.0);",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
        let d =
            parse("layout(local_size_x=1) in; const vec2 workGroupsRender=vec2(3e38);").unwrap();
        assert!(d.groups([100, 100]).is_none());
    }
}
