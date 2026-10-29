import SwiftUI
import UIKit

final class Kit {
    static let bundle = Bundle(for: Kit.self)
    static func image(_ name: String) -> Image { Image(name, bundle: bundle) }
}

/// Semantic UIKit colors adapt to appearance, contrast and accessibility settings.
/// Legacy type helper names remain so all launcher pages share the native typography.
enum Theme {
    static let background = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? UIColor(red: 0.035, green: 0.055, blue: 0.075, alpha: 1) : UIColor(red: 0.94, green: 0.96, blue: 0.97, alpha: 1)
    })
    static let surface = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? UIColor(red: 0.075, green: 0.10, blue: 0.125, alpha: 1) : .white
    })
    static let surfaceSunken = Color(uiColor: .tertiarySystemGroupedBackground)
    static let surfaceDisabled = Color(uiColor: .tertiarySystemFill)
    static let border = Color(uiColor: .separator).opacity(0.28)
    static let accent = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? UIColor(red: 0.34, green: 0.85, blue: 0.79, alpha: 1) : UIColor(red: 0.025, green: 0.40, blue: 0.44, alpha: 1)
    })
    static let accentEdge = accent
    static let accentBright = accent
    static let accentText = accent
    static let notice = accent.opacity(0.12)
    static let noticeBorder = accent.opacity(0.18)
    static let ready = Color.green
    static let warn = Color.orange
    static let danger = Color.red
    static let splash = Color.yellow
    static let splashShadow = Color.black.opacity(0.3)
    static let text = Color.primary
    static let textMuted = Color.secondary
    static let textFaint = Color(uiColor: .secondaryLabel)

    enum PixelWeight { case medium, semiBold, bold }
    enum SansWeight { case regular, medium, semiBold }
    static func pixel(_ size: CGFloat, _ weight: PixelWeight = .semiBold) -> Font {
        native(size).weight(weight == .bold ? .bold : weight == .medium ? .medium : .semibold)
    }
    static func sans(_ size: CGFloat, _ weight: SansWeight = .regular) -> Font {
        native(size).weight(weight == .semiBold ? .semibold : weight == .medium ? .medium : .regular)
    }
    static func mono(_ size: CGFloat) -> Font { native(size).monospaced() }
    private static func native(_ size: CGFloat) -> Font {
        switch size {
        case 28...: return .largeTitle
        case 22..<28: return .title2
        case 18..<22: return .title3
        case 16..<18: return .body
        case 14..<16: return .callout
        case 12..<14: return .footnote
        default: return .caption
        }
    }
    static func registerFonts() { /* Uses system fonts; no font registration needed. */ }
}

extension Color {
    init(hex: UInt32, opacity: Double = 1) {
        self.init(.sRGB, red: Double((hex >> 16) & 255) / 255,
                  green: Double((hex >> 8) & 255) / 255, blue: Double(hex & 255) / 255, opacity: opacity)
    }
}

extension View {
    /// Native Liquid Glass for controls on iOS 26+, system materials on older supported releases.
    @ViewBuilder func launcherGlass() -> some View {
        if #available(iOS 26.0, *) {
            self.glassEffect(.regular, in: .rect(cornerRadius: 24))
        } else {
            self.background(.regularMaterial, in: RoundedRectangle(cornerRadius: 24))
        }
    }
    @ViewBuilder func launcherPrimaryButton() -> some View {
        if #available(iOS 26.0, *) { self.buttonStyle(.glassProminent) }
        else { self.buttonStyle(.borderedProminent) }
    }
    @ViewBuilder func launcherSecondaryButton() -> some View {
        if #available(iOS 26.0, *) { self.buttonStyle(.glass) }
        else { self.buttonStyle(.bordered) }
    }
}

private struct AccountCircleMaterial: ViewModifier {
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @ViewBuilder func body(content: Content) -> some View {
        if reduceTransparency {
            content.background(Theme.surface, in: Circle()).overlay(Circle().strokeBorder(Theme.border))
        } else if #available(iOS 26.0, *) {
            content.glassEffect(.regular.interactive(), in: .circle)
        } else {
            content.background(.regularMaterial, in: Circle()).overlay(Circle().strokeBorder(Theme.border))
        }
    }
}
extension View {
    func accountCircleGlass() -> some View { modifier(AccountCircleMaterial()) }
}
