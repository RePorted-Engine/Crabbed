import Foundation

/// Simulator/UI-preview backend for the same schema as options-get/options-set. Not linked into the game.
/// EngineOptions.json is emitted by `cargo run --example native_launcher_options --no-default-features`.
enum PreviewOptions {
    private static let lock = NSLock()
    private static var options: [String: [String: Any]] = {
        guard let url = Bundle.main.url(forResource: "EngineOptions", withExtension: "json"),
              let data = try? Data(contentsOf: url),
              let json = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
        else { return [:] }
        return json["options"] as? [String: [String: Any]] ?? [:]
    }()

    static func reply(_ line: String) -> [String: Any]? {
        let parts = line.split(separator: " ").map(String.init)
        guard let command = parts.first, command == "options-get" || command == "options-set" else { return nil }
        lock.lock()
        defer { lock.unlock() }
        switch ProcessInfo.processInfo.environment["PREVIEW_OPTIONS_RESPONSE"] {
        case "empty": return ["options": [:]]
        case "error": return ["error": "The settings preview source is unavailable."]
        default: break
        }
        if command == "options-get" {
            let keys = parts.count == 1 ? Array(options.keys) : Array(parts.dropFirst())
            var selected: [String: [String: Any]] = [:]
            for key in keys {
                guard let option = options[key] else { return ["error": "unknown option \(key)"] }
                selected[key] = option
            }
            return ["options": selected]
        }
        guard parts.count == 3, var option = options[parts[1]] else { return ["error": "Unknown setting."] }
        let value = parts[2]
        switch option["type"] as? String {
        case "bool":
            guard value == "true" || value == "false" else { return ["error": "Expected true or false."] }
            option["value"] = value == "true"
        case "enum":
            guard (option["choices"] as? [String] ?? []).contains(value) else { return ["error": "Invalid choice."] }
            option["value"] = value
        case "int", "double", "float":
            guard let number = Double(value), number.isFinite,
                  number >= (option["min"] as? Double ?? -.infinity),
                  number <= (option["max"] as? Double ?? .infinity) else { return ["error": "Value is outside the supported range."] }
            if option["type"] as? String == "int" {
                guard let integer = Int(value) else { return ["error": "Expected an integer."] }
                option["value"] = integer
            } else { option["value"] = number }
        default: return ["error": "Unsupported setting type."]
        }
        options[parts[1]] = option
        return ["ok": true, "key": parts[1], "option": option]
    }
}
