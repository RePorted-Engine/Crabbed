import SwiftUI

/// Minecraft legacy formatting in server descriptions. Formatting codes are consumed, never displayed.
/// A color changes also reset decoration; §r returns to the caller's semantic foreground color.
enum MinecraftFormatting {
    struct Style: Equatable {
        var color: UInt32?
        var bold = false
        var italic = false
        var underline = false
        var strike = false
        var obfuscated = false
    }
    struct Run: Equatable { var text: String; var style: Style }
    static func runs(_ source: String) -> [Run] {
        let chars = Array(source)
        let palette: [UInt32] = [0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xFFAA00, 0xAAAAAA,
                                 0x555555, 0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF]
        var result: [Run] = []
        var style = Style()
        var buffer = ""
        var i = 0
        func flush() {
            if !buffer.isEmpty { result.append(Run(text: buffer, style: style)); buffer = "" }
        }
        while i < chars.count {
            guard chars[i] == "§", i + 1 < chars.count else { buffer.append(chars[i]); i += 1; continue }
            let code = String(chars[i + 1]).lowercased()
            // Bungee/modern legacy RGB: §x§R§R§G§G§B§B.
            if code == "x", i + 13 < chars.count {
                var hex = ""
                for offset in stride(from: 2, through: 12, by: 2) {
                    guard chars[i + offset] == "§", chars[i + offset + 1].isHexDigit else { hex = ""; break }
                    hex.append(chars[i + offset + 1])
                }
                if hex.count == 6, let color = UInt32(hex, radix: 16) {
                    flush(); style = Style(color: color); i += 14; continue
                }
            }
            if let color = Int(code, radix: 16), (0...15).contains(color) {
                flush(); style = Style(color: palette[color]); i += 2; continue
            }
            if ["k", "l", "m", "n", "o", "r"].contains(code) {
                flush()
                switch code {
                case "k": style.obfuscated = true
                case "l": style.bold = true
                case "m": style.strike = true
                case "n": style.underline = true
                case "o": style.italic = true
                default: style = Style()
                }
                i += 2; continue
            }
            // Unknown sequences are ordinary text; do not eat a literal section sign or the next letter.
            buffer.append(chars[i]); i += 1
        }
        flush()
        return result
    }
    static func plain(_ source: String) -> String { runs(source).map(\.text).joined() }
    static func text(_ source: String, defaultColor: Color) -> Text {
        runs(source).reduce(Text("")) { text, run in
            let visible = run.style.obfuscated ? String(run.text.map { $0.isWhitespace ? $0 : "▪" }) : run.text
            var part = Text(verbatim: visible).foregroundColor(run.style.color.map { Color(hex: $0) } ?? defaultColor)
            if run.style.bold { part = part.bold() }
            if run.style.italic { part = part.italic() }
            if run.style.underline { part = part.underline() }
            if run.style.strike { part = part.strikethrough() }
            return text + part
        }
    }
}
