import SwiftUI
import UIKit

/// A labelled text field in the launcher's look.
struct Field: View {
    let title: String
    let placeholder: String
    @Binding var text: String
    var secure = false
    var keyboard: UIKeyboardType = .URL

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title.localizedCapitalized).font(.subheadline).foregroundStyle(Theme.textMuted)
            Group {
                if secure {
                    SecureField(placeholder, text: $text)
                } else {
                    TextField(placeholder, text: $text)
                        .keyboardType(keyboard)
                }
            }
            .textInputAutocapitalization(.never)
            .autocorrectionDisabled()
            .font(Theme.sans(14))
            .foregroundStyle(Theme.text)
            .padding(.horizontal, 12)
            .frame(minHeight: 44)
            .background(Theme.surfaceSunken, in: RoundedRectangle(cornerRadius: 8))
            .overlay(RoundedRectangle(cornerRadius: 8).strokeBorder(Theme.border))
        }
    }
}

/// System button styles provide native interaction, disabled state and Liquid Glass.
struct PrimaryButton: View {
    let title: String
    var icon: String?
    let action: () -> Void
    var body: some View {
        Button { Haptics.tap(); action() } label: {
            Label(title, systemImage: icon ?? "arrow.right")
                .font(.headline).frame(maxWidth: .infinity, minHeight: 36)
        }
        .controlSize(.large)
        .launcherPrimaryButton()
    }
}

struct SecondaryButton: View {
    let title: String
    var icon: String?
    let action: () -> Void
    var body: some View {
        Button { Haptics.tap(); action() } label: {
            Label(title, systemImage: icon ?? "arrow.right")
                .font(.body).frame(maxWidth: .infinity, minHeight: 32)
        }
        .controlSize(.large)
        .launcherSecondaryButton()
    }
}

/// Buttons sink a little under a finger, as the game's do when pressed.
struct PressStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .brightness(configuration.isPressed ? -0.06 : 0)
            .animation(.easeOut(duration: 0.12), value: configuration.isPressed)
    }
}

/// A section's heading, in the pixel face, with an optional action at its end.
struct SectionHeader<Trailing: View>: View {
    let title: String
    @ViewBuilder var trailing: () -> Trailing

    var body: some View {
        HStack {
            Text(title).font(Theme.pixel(20)).foregroundStyle(Theme.text)
            Spacer()
            trailing()
        }
        .padding(.top, 8)
    }
}

extension SectionHeader where Trailing == EmptyView {
    init(title: String) {
        self.init(title: title) { EmptyView() }
    }
}

/// A page's own heading under the header bar.
struct PageTitle: View {
    let title: String
    var subtitle: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(Theme.pixel(30)).foregroundStyle(Theme.text)
            if let subtitle {
                Text(subtitle).font(Theme.sans(12)).foregroundStyle(Theme.textMuted)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// An empty list's word, in a sunken box.
struct EmptyNote: View {
    let icon: String
    let text: String

    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: icon).font(.system(size: 26)).foregroundStyle(Theme.textFaint)
            Text(text).font(Theme.sans(13)).foregroundStyle(Theme.textMuted).multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(24)
        .background(Theme.surfaceSunken, in: RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Theme.border))
    }
}

extension View {
    /// The launcher's card: a raised surface with a hairline border.
    func card(radius: CGFloat = 12) -> some View {
        background(Theme.surface, in: RoundedRectangle(cornerRadius: radius))

    }

    /// A page's width and margins, as the home screen has them.
    func pageColumn() -> some View {
        padding(.horizontal, 20)
            .padding(.top, 12)
            .padding(.bottom, 28)
            .frame(maxWidth: 520)
            .frame(maxWidth: .infinity)
    }
}
