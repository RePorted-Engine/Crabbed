import GameController
import Network
import SwiftUI

/// What the phone is ready for, for the launcher's chips: a controller, a network, and room for worlds.
@MainActor
final class DeviceStatus: ObservableObject {
    @Published private(set) var controller: String?
    @Published private(set) var network: String?
    @Published private(set) var freeBytes: Int64?

    private let monitor = NWPathMonitor()
    private var observers: [NSObjectProtocol] = []

    init() {
        monitor.pathUpdateHandler = { [weak self] path in
            let network: String?
            if path.status != .satisfied {
                network = nil
            } else if path.usesInterfaceType(.wifi) {
                network = "Wi-Fi"
            } else if path.usesInterfaceType(.cellular) {
                network = "Cellular"
            } else {
                network = "Online"
            }
            // A constant, not the weak var: a concurrent closure may not capture a var.
            let status = self
            DispatchQueue.main.async { status?.network = network }
        }
        monitor.start(queue: DispatchQueue(label: "dev.horizon.launcher.network"))
        let center = NotificationCenter.default
        for name in [Notification.Name.GCControllerDidConnect, .GCControllerDidDisconnect] {
            observers.append(center.addObserver(forName: name, object: nil, queue: .main) { [weak self] _ in
                let status = self
                Task { @MainActor in status?.readController() }
            })
        }
        readController()
        readStorage()
    }

    deinit {
        monitor.cancel()
        for observer in observers {
            NotificationCenter.default.removeObserver(observer)
        }
    }

    private func readController() {
        controller = GCController.controllers().first.map { $0.vendorName ?? "Controller" }
    }

    func readStorage() {
        let values = try? GameFolder.documents.resourceValues(forKeys: [.volumeAvailableCapacityForImportantUsageKey])
        freeBytes = values?.volumeAvailableCapacityForImportantUsage
    }
}

/// One status chip: a coloured square, as the hero's version badge has, and a word or two.
struct Chip: View {
    enum Tone {
        case good, warn, bad, quiet

        var color: Color {
            switch self {
            case .good: return Theme.ready
            case .warn: return Theme.warn
            case .bad: return Theme.danger
            case .quiet: return Theme.textFaint
            }
        }
    }

    let tone: Tone
    let text: String
    var icon: String?

    var body: some View {
        HStack(spacing: 6) {
            if let icon {
                Image(systemName: icon).font(.system(size: 10, weight: .semibold)).foregroundStyle(tone.color)
            } else {
                Rectangle().fill(tone.color).frame(width: 5, height: 5)
            }
            Text(text).font(Theme.mono(10)).foregroundStyle(Theme.text).lineLimit(1)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 6)
        .background(Theme.surfaceSunken, in: RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).strokeBorder(Theme.border))
    }
}

/// The row of chips under the hero: the game, the controller, the network and the storage.
struct StatusChips: View {
    @ObservedObject var link: GameLink
    @ObservedObject var device: DeviceStatus

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
                game
                if let controller = device.controller {
                    Chip(tone: .good, text: controller, icon: "gamecontroller.fill")
                } else {
                    Chip(tone: .quiet, text: "Touch", icon: "hand.tap.fill")
                }
                if let network = device.network {
                    Chip(tone: .good, text: network, icon: "wifi")
                } else {
                    Chip(tone: .warn, text: "Offline", icon: "wifi.slash")
                }
                if let free = device.freeBytes {
                    Chip(tone: free < 1_000_000_000 ? .warn : .quiet,
                         text: "\(ByteCountFormatter.string(fromByteCount: free, countStyle: .file)) free",
                         icon: "internaldrive")
                }
            }
        }
    }

    @ViewBuilder private var game: some View {
        switch link.phase {
        case .title, .world:
            Chip(tone: .good, text: "Minecraft \(link.version)")
        case let .loading(progress):
            Chip(tone: .warn, text: "Loading \(Int(progress * 100))%")
        case .needsFiles:
            Chip(tone: .bad, text: "No game files")
        case .waiting:
            if GameFolder.hasGameFiles {
                Chip(tone: .good, text: "Minecraft \(link.version)")
            } else {
                Chip(tone: .bad, text: "No game files")
            }
        case .unknown:
            Chip(tone: .quiet, text: "Starting")
        }
    }
}
