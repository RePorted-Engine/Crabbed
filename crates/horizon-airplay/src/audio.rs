//! The phone's audio session, for the game's sound (cpal's RemoteIO output): playback, so the game is heard with
//! the ring/silent switch on as a game the player opened to play should be, mixed with whatever else is playing
//! rather than stopping it. iOS's default (solo ambient) would go silent with the switch.
//!
//! shortcut: no interruption observer, so after a phone call the session stays inactive until the next start;
//! add `AVAudioSessionInterruptionNotification` handling when that is reported.

/// Set the session up: once, before the game opens its audio stream. Nothing elsewhere.
pub fn start() {
    #[cfg(target_os = "ios")]
    ios::start();
}

#[cfg(target_os = "ios")]
mod ios {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject, Bool};
    use objc2_foundation::NSString;

    // AVAudioSession lives in AVFAudio; nothing else in the game links it.
    #[link(name = "AVFAudio", kind = "framework")]
    unsafe extern "C" {}

    /// `AVAudioSessionCategoryOptionMixWithOthers`.
    const MIX_WITH_OTHERS: usize = 1;

    pub fn start() {
        let Some(class) = AnyClass::get(c"AVAudioSession") else {
            eprintln!("Horizon: no AVAudioSession");
            return;
        };
        // `AVAudioSessionCategoryPlayback`'s value is its own name.
        let category = NSString::from_str("AVAudioSessionCategoryPlayback");
        let no_error: *mut *mut AnyObject = std::ptr::null_mut();
        // SAFETY: documented AVAudioSession selectors with their documented argument types; the shared
        // session lives for the whole process.
        let ok = unsafe {
            let session: *mut AnyObject = msg_send![class, sharedInstance];
            if session.is_null() {
                return;
            }
            let category: Bool = msg_send![
                session,
                setCategory: &*category,
                withOptions: MIX_WITH_OTHERS,
                error: no_error
            ];
            let active: Bool = msg_send![session, setActive: Bool::YES, error: no_error];
            category.as_bool() && active.as_bool()
        };
        if !ok {
            eprintln!("Horizon: the audio session would not start");
        }
    }
}
