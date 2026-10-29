import SwiftUI
import UIKit

private let previewSubmit: HorizonSubmit = { pointer, context, done in
    let line = pointer.map { String(cString: $0) } ?? ""
    let response: [String: Any]
    if let options = PreviewOptions.reply(line) { response = options }
    else { switch line {
    case "launcher-state": response = ["phase": "title", "version": "26.3"]
    case "worlds":
        response = ["worlds": [
            ["folder": "meadow", "name": "Meadow Valley", "mode": "survival", "version": "26.3", "last_played": Date().timeIntervalSince1970 * 1000 - 3600000],
            ["folder": "creative", "name": "Creative Workshop", "mode": "creative", "version": "26.3", "last_played": Date().timeIntervalSince1970 * 1000 - 86400000],
            ["folder": "islands", "name": "The Islands", "mode": "survival", "version": "26.3", "last_played": Date().timeIntervalSince1970 * 1000 - 172800000]
        ]]
    case "servers": response = ["servers": [["index": 0, "name": "Hypixel", "address": "mc.hypixel.net"]]]
    case "server-ping mc.hypixel.net": response = ["motd": "§6§lHypixel Network §8[§c1.8 / 26.3§8]\n§f§lSKYBLOCK §r§dHalloween Event", "online": 9813, "max": 200000, "version": "Java Edition", "ms": 79]
    default: response = ["ok": true]
    }
    }
    let data = try! JSONSerialization.data(withJSONObject: response)
    String(data: data, encoding: .utf8)!.withCString { done(context, $0) }
}

@main
struct LauncherPreview: App {
    @State private var previewNotice = false
    init() {
        for key in ["HORIZON_HOME", "HORIZON_LAUNCHER_HOME", "XDG_DATA_HOME", "XDG_CONFIG_HOME",
                    "HORIZON_LAUNCHER_TRASH", "HORIZON_LAUNCHER_CLIPBOARD", "HORIZON_LAUNCHER_DESKTOP"] {
            if let value = ProcessInfo.processInfo.environment[key] {
                try? FileManager.default.createDirectory(atPath: value, withIntermediateDirectories: true)
            }
        }
        Theme.registerFonts()
        Bridge.submit = previewSubmit
        if ProcessInfo.processInfo.environment["PREVIEW_SIGNED_IN"] == "1" {
            let profile = MicrosoftAccount.Profile(name: "SkyExplorer", uuid: "00000000000000000000000000000000")
            UserDefaults.standard.set(try! JSONEncoder().encode(profile), forKey: "microsoftProfile")
        } else if ProcessInfo.processInfo.environment["PREVIEW_SIGNED_IN"] == "0" {
            UserDefaults.standard.removeObject(forKey: "microsoftProfile")
        }
        if ProcessInfo.processInfo.environment["PREVIEW_SIGNED_IN"] != nil {
            try? "name:OfflinePlayer\n".write(to: GameFolder.documents.appendingPathComponent("horizon-player.txt"), atomically: true, encoding: .utf8)
        }
    }
    var body: some Scene {
        WindowGroup {
            HomeView(onPlay: { previewNotice = true })
                .alert("Launcher UI preview", isPresented: $previewNotice) {
                    Button("OK", role: .cancel) {}
                } message: { Text("The worlds and server entry here are sample data for reviewing the interface. This preview does not start the game.") }
                .frame(width: ProcessInfo.processInfo.environment["PREVIEW_WIDTH"].flatMap(Double.init).map { CGFloat($0) })
        }
    }
}
