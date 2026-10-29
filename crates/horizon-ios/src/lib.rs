//! Horizon's simulation core, reached from an iPhone app's Objective-C.
//!
//! The crate exists to be linked, not to be a dependency: it builds as a `staticlib` for
//! `aarch64-apple-ios`, and the app calls [`horizon_probe`] to run four of Horizon's own
//! subsystems on the device and read their answers back as text. Nothing here touches a window, a
//! GPU, a resource pack or the player's Minecraft files, which is what makes it the first thing
//! that can honestly run on a phone: the game's logic, without the game's assets.

use std::os::raw::c_char;
use std::ptr;

use horizon_core::jmath;
use horizon_core::paletted::{PalettedContainer, Strategy};
use horizon_core::random::{LegacyRandom, RandomSource, XoroshiroRandom};
use horizon_world::nbt::{JavaString, Limits, NamedTag, Tag};

/// The largest report the probe can write, including the closing NUL.
const MAX_REPORT: usize = 4096;

/// Vanilla's own numbers, pinned by `horizon_core::version`.
fn identity(out: &mut String) {
    use horizon_core::version;
    out.push_str(&format!(
        "Horizon targets Minecraft Java {}\nprotocol {}  data version {}  Java {}\n\n",
        version::VANILLA_VERSION,
        version::PROTOCOL_VERSION,
        version::DATA_VERSION,
        version::JAVA_VERSION,
    ));
}

/// `horizon_core::random`: vanilla's Xoroshiro128** draws, in vanilla's order.
fn draws(out: &mut String) {
    let mut random = XoroshiroRandom::new(12_345);
    let ints: Vec<i32> = (0..5)
        .map(|_| random.next_int())
        .map(|value| value >> 20)
        .collect();
    let (low, high) = random.state_words();
    out.push_str("xoroshiorepeat draws from seed 12345\n");
    out.push_str(&format!("  nextInt>>20  {:?}\n", ints));
    out.push_str(&format!("  state        {low:#018x} {high:#018x}\n"));
    let mut legacy = LegacyRandom::new(12_345);
    out.push_str(&format!(
        "  java.util.Random nextInt(360) {}\n\n",
        legacy.next_int_between_inclusive(0, 359).unwrap_or(-1)
    ));
}

/// One `jmath` result, with the bits that make it exact.
fn show(out: &mut String, name: &str, value: f64) {
    out.push_str(&format!(
        "  {name} = {value:.17}  bits {:#018x}\n",
        value.to_bits()
    ));
}

/// `horizon_core::jmath`: the HotSpot-exact `Math`, bit for bit.
fn maths(out: &mut String) {
    out.push_str("jmath, HotSpot-exact\n");
    show(out, "sin(1.0)", jmath::sin(1.0));
    show(out, "cos(1.0)", jmath::cos(1.0));
    show(out, "atan2(1.0, 3.0)", jmath::atan2(1.0, 3.0));
    show(out, "exp(0.5)", jmath::exp(0.5));
    out.push('\n');
}

/// `horizon_core::paletted`: a section of 4096 block states, packed and read back.
fn sections(out: &mut String) {
    let strategy = Strategy::blocks(13);
    // Three distinct states, the way a real section looks: air, then a floor, then a wall.
    let values: Vec<u32> = (0..strategy.entries())
        .map(|i| match i {
            0..=100 => 0,
            101..=200 => 1,
            _ => 2,
        })
        .collect();
    let container = PalettedContainer::from_values(strategy, &values);
    let packed = container.pack();
    let restored =
        PalettedContainer::from_packed(strategy, packed.palette.clone(), packed.data.as_deref());
    let mut back = vec![0u32; strategy.entries()];
    let matches = match restored {
        Ok(restored) => {
            restored.unpack(&mut back);
            back == values && restored.palette_len() == packed.palette.len()
        }
        Err(_) => false,
    };
    out.push_str("paletted section, 4096 block states\n");
    out.push_str(&format!(
        "  bits {}  palette {}\n",
        container.bits(),
        container.palette_len()
    ));
    out.push_str(&format!(
        "  packed to {} words, read back equal: {}\n\n",
        packed.data.map_or(0, |words| words.len()),
        matches
    ));
}

/// `horizon_world::nbt`: the save format, written and parsed again in memory.
fn level_dat(out: &mut String) {
    let root = Tag::compound([
        ("SpawnX", Tag::Int(64)),
        ("SpawnY", Tag::Int(71)),
        ("SpawnZ", Tag::Int(-128)),
        ("SizeOnDisk", Tag::Long(1_048_576)),
        ("Version", Tag::Int(5023)),
        ("GameType", Tag::Int(1)),
        ("generatorName", Tag::String("minecraft:overworld".into())),
    ]);
    let named = NamedTag {
        name: JavaString::default(),
        tag: root,
    };
    let written = horizon_world::nbt::write_named(&named, Limits::default()).ok();
    let read_back = match written.as_ref() {
        Some(bytes) => horizon_world::nbt::read_named(bytes, Limits::default())
            .map(|(parsed, used)| used == bytes.len() && parsed == named)
            .unwrap_or(false),
        None => false,
    };
    out.push_str("level.dat root compound\n");
    out.push_str(&format!(
        "  wrote {} bytes, parsed back identical: {}\n",
        written.map_or(0, |bytes| bytes.len()),
        read_back
    ));
}

/// What the app shows: the report from the four probes above, as NUL-terminated UTF-8.
fn report() -> String {
    let mut out = String::new();
    identity(&mut out);
    draws(&mut out);
    maths(&mut out);
    sections(&mut out);
    level_dat(&mut out);
    out
}

/// Runs Horizon's random draws, math, palette packing and NBT codec on this device, writing the
/// result into `out` as NUL-terminated UTF-8 and returning the byte count without the NUL. A
/// negative return is an error code, never a length: an app that printed one as a count would report
/// that an honest simulation core had written nothing at all.
///
/// Returns -1 when `cap` is under `MAX_REPORT` or `out` is null, and -2 when the report would not
/// have fitted, which cannot happen today but keeps the caller honest about the buffer it passed.
///
/// # Safety
///
/// `out` must point at `cap` writable bytes, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn horizon_probe(out: *mut c_char, cap: usize) -> isize {
    // A buffer shorter than MAX_REPORT is not an answer: the caller asked for a report it cannot
    // hold, so -1 sends it back to horizon_probe_capacity() to size the buffer properly.
    if out.is_null() || cap < MAX_REPORT {
        return -1;
    }
    let text = report();
    if text.len() + 1 > cap {
        return -2;
    }
    // SAFETY: the caller guarantees `out` has `cap` writable bytes, `text` is `cap - 1` bytes or
    // fewer, and the two ranges do not overlap: `out` is the caller's buffer, not this String's.
    unsafe {
        ptr::copy_nonoverlapping(text.as_ptr().cast(), out, text.len());
        *out.add(text.len()) = 0;
    }
    text.len() as isize
}

/// The smallest buffer [`horizon_probe`] will accept, so the caller can allocate once and check.
#[unsafe(no_mangle)]
pub extern "C" fn horizon_probe_capacity() -> usize {
    MAX_REPORT
}

/// The report, as a Rust string, for the tests and for anything that links the `rlib`.
pub fn probe_text() -> String {
    report()
}

/// Round-trips the C entry point through a buffer, the way the app calls it.
#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::ptr::null_mut;

    use super::{MAX_REPORT, horizon_probe, horizon_probe_capacity, probe_text};

    #[test]
    fn probe_reports_the_capacity_it_accepts() {
        assert_eq!(horizon_probe_capacity(), MAX_REPORT);
    }

    #[test]
    fn probe_writes_a_terminated_report() {
        let mut buffer = vec![0u8; MAX_REPORT];
        let written = unsafe { horizon_probe(buffer.as_mut_ptr().cast(), buffer.len()) };
        assert!(written > 0, "the probe wrote {written}");
        let text = CStr::from_bytes_until_nul(&buffer[..(written as usize) + 1])
            .expect("NUL-terminated")
            .to_str()
            .expect("UTF-8");
        assert_eq!(text, probe_text());
        assert!(text.contains("protocol 777"));
        assert!(text.contains("read back equal: true"));
        assert!(text.contains("parsed back identical: true"));
    }

    #[test]
    fn probe_refuses_a_small_buffer() {
        let mut buffer = vec![0u8; MAX_REPORT - 1];
        assert_eq!(
            unsafe { horizon_probe(buffer.as_mut_ptr().cast(), buffer.len()) },
            -1
        );
        assert_eq!(unsafe { horizon_probe(null_mut(), MAX_REPORT) }, -1);
    }
}
