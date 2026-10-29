import AVFoundation

/// The phone's audio session, for the game's sound (cpal's RemoteIO output, the `audio` feature): playback,
/// so the game is heard with the ring/silent switch on as a game the player opened to play should be, mixed
/// with whatever else is playing rather than stopping it. Activated when the game is shown and deactivated when its engine returns to idle.
enum GameAudio {
    private static var observer: NSObjectProtocol?

    static func stop() {
        if let observer { NotificationCenter.default.removeObserver(observer) }
        observer = nil
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    static func start() {
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setCategory(.playback, mode: .default, options: [.mixWithOthers])
            try session.setActive(true)
        } catch {
            NSLog("Horizon: the audio session would not start: \(error)")
        }
        guard observer == nil else { return }
        // A call or an alarm takes the session; take it back when it ends.
        observer = NotificationCenter.default.addObserver(
            forName: AVAudioSession.interruptionNotification, object: session, queue: .main
        ) { note in
            let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
            if raw.flatMap(AVAudioSession.InterruptionType.init(rawValue:)) == .ended {
                try? session.setActive(true)
            }
        }
    }
}
