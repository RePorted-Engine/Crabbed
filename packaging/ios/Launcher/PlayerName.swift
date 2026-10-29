import Foundation

/// The offline player name, kept where the game keeps it: `name:<name>` in `Documents/horizon-player.txt`
/// (crates/horizon-client/src/offline_name.rs). The game reads the file whenever it joins a world or a
/// server, so a name saved here is the one it plays with from the next join on.
final class PlayerName: ObservableObject {
    /// The kept name; nil until the game has made one or the player has chosen one.
    @Published private(set) var name: String?

    init() {
        name = Self.load()
    }

    /// `StringUtil.isValidPlayerName` as the game checks it (`lan::conn::valid_name`): 1 to 16 characters,
    /// each printable ASCII and not a space.
    static func isValid(_ name: String) -> Bool {
        (1...16).contains(name.unicodeScalars.count)
            && name.unicodeScalars.allSatisfy { $0.value > 32 && $0.value < 127 }
    }

    /// Keep `name` for the game. False, and nothing written, when it is not a valid name or the file can't be
    /// written.
    @discardableResult
    func save(_ name: String) -> Bool {
        guard Self.isValid(name), let file = Self.file else {
            return false
        }
        do {
            try "name:\(name)\n".write(to: file, atomically: true, encoding: .utf8)
        } catch {
            return false
        }
        self.name = name
        return true
    }

    private static var file: URL? {
        FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?
            .appendingPathComponent("horizon-player.txt")
    }

    private static func load() -> String? {
        guard let file, let text = try? String(contentsOf: file, encoding: .utf8) else {
            return nil
        }
        return text.split(whereSeparator: \.isNewline)
            .lazy
            .compactMap { line -> String? in
                let line = line.hasSuffix("\r") ? line.dropLast() : line[...]
                return line.hasPrefix("name:") ? String(line.dropFirst(5)) : nil
            }
            .first(where: isValid)
    }
}
