import SwiftUI
import UIKit

/// The multiplayer server list (servers.dat, the game's own): each server pinged for its message, players and
/// latency, tap to join, hold to remove; Add Server and Direct Connection above.
struct ServersPage: View {
    @ObservedObject var link: GameLink
    @ObservedObject var account: MicrosoftAccount
    @State private var adding = false
    @State private var direct = ""
    @State private var removing: Server?

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 10) {
                Field(title: "DIRECT CONNECTION", placeholder: "play.example.com or 192.168.1.20:25565",
                      text: $direct)
                PrimaryButton(title: "Join Server", icon: "arrow.right") {
                    Task { await link.join(direct) }
                }
                .disabled(direct.trimmingCharacters(in: .whitespaces).isEmpty || link.launching != nil)
            }
            .padding(16)
            .card()

            SectionHeader(title: "Saved servers") {
                Button {
                    Haptics.tap()
                    adding = true
                } label: {
                    Label("Add", systemImage: "plus").font(Theme.sans(13, .semiBold))
                        .foregroundStyle(Theme.accentText).frame(minHeight: 44)
                }
                .disabled(!link.canList)
            }
            if !link.canList {
                EmptyNote(icon: "hourglass", text: "Your servers show here in a moment.")
            } else if link.servers.isEmpty {
                EmptyNote(icon: "globe", text: "No servers yet. Add one to keep it here.")
            } else {
                VStack(spacing: 0) {
                    ForEach(link.servers) { server in
                        ServerRow(server: server, status: link.statuses[server.address]) {
                            Task { await link.join(server.address) }
                        }
                        .disabled(link.launching != nil)
                        .contextMenu {
                            Button {
                                Task { await link.join(server.address) }
                            } label: { Label("Join", systemImage: "play.fill") }
                            Button {
                                UIPasteboard.general.string = server.address
                            } label: { Label("Copy Address", systemImage: "doc.on.doc") }
                            Button(role: .destructive) {
                                removing = server
                            } label: { Label("Remove", systemImage: "trash") }
                        }
                        // Again on every reload, which a pull down asks for.
                        .task(id: "\(server.id) \(link.serverRound)") { await link.ping(server) }
                        if server != link.servers.last {
                            Divider().overlay(Theme.border).padding(.leading, 70)
                        }
                    }
                }
                .card()
                Text("Hold a server to copy its address or remove it. Pull down to refresh.")
                    .font(Theme.sans(11)).foregroundStyle(Theme.textFaint)
            }
            if let launching = link.launching {
                HStack(spacing: 8) {
                    ProgressView().tint(Theme.accentText)
                    Text(launching).font(Theme.sans(12)).foregroundStyle(Theme.textMuted)
                    Spacer()
                    Button("Cancel") { link.cancelLaunch() }.font(Theme.sans(12, .semiBold))
                }
            }
        }
        .sheet(isPresented: $adding) {
            AddServerSheet { name, address in
                adding = false
                Task { await link.addServer(name: name, address: address) }
            }
        }
        .confirmationDialog(
            "Remove \(removing?.name ?? "this server")?",
            isPresented: Binding(get: { removing != nil }, set: { if !$0 { removing = nil } }),
            titleVisibility: .visible
        ) {
            Button("Remove", role: .destructive) {
                if let server = removing {
                    Task { await link.removeServer(server) }
                }
            }
        }
    }
}

private struct ServerRow: View {
    let server: Server
    let status: ServerStatus?
    let join: () -> Void

    var body: some View {
        Button(action: join) {
            HStack(alignment: .top, spacing: 12) {
                icon
                VStack(alignment: .leading, spacing: 3) {
                    HStack {
                        MinecraftFormatting.text(server.name, defaultColor: Theme.text).font(.headline).lineLimit(1)
                        Spacer(minLength: 4)
                        signal
                    }
                    MinecraftFormatting.text(detail, defaultColor: detailColor).font(.subheadline).lineLimit(3)
                        .accessibilityLabel(MinecraftFormatting.plain(detail))
                        .multilineTextAlignment(.leading)
                    if let status, status.failed == nil {
                        MinecraftFormatting.text("\(status.online)/\(status.max) players · \(status.version)", defaultColor: Theme.textFaint)
                            .font(Theme.mono(9)).foregroundStyle(Theme.textFaint).lineLimit(1)
                    }
                }
            }
            .padding(12)
            .contentShape(Rectangle())
        }
        .buttonStyle(PressStyle())
    }

    private var detail: String {
        guard let status else { return "Pinging \(server.address)…" }
        if status.failed != nil { return "Can't connect to server" }
        return status.motd.isEmpty ? server.address : status.motd
    }

    private var detailColor: Color {
        status?.failed != nil ? Theme.danger : Theme.textMuted
    }

    /// The server's own 64x64 icon (the one it last sent, or the one servers.dat kept), else a globe.
    private var icon: some View {
        let base64 = status?.icon ?? server.icon
        let image = base64.flatMap { Data(base64Encoded: $0, options: .ignoreUnknownCharacters) }.flatMap(UIImage.init(data:))
        return Group {
            if let image {
                Image(uiImage: image).resizable().interpolation(.none)
            } else {
                ZStack {
                    Theme.surfaceSunken
                    Image(systemName: "globe").foregroundStyle(Theme.textFaint)
                }
            }
        }
        .frame(width: 46, height: 46)
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .overlay(RoundedRectangle(cornerRadius: 6).strokeBorder(Theme.border))
    }

    /// The server list's five bars: more for a quicker answer, a cross for none.
    private var signal: some View {
        let bars: Int
        let color: Color
        if let status {
            if status.failed != nil {
                bars = 0
                color = Theme.danger
            } else {
                // ServerSelectionList.OnlineServerEntry: <150, <300, <600, <1000 ms, else one bar.
                switch status.ms {
                case ..<150: bars = 5
                case ..<300: bars = 4
                case ..<600: bars = 3
                case ..<1000: bars = 2
                default: bars = 1
                }
                color = Theme.ready
            }
        } else {
            bars = 0
            color = Theme.textFaint
        }
        return HStack(alignment: .bottom, spacing: 2) {
            if status?.failed != nil {
                Image(systemName: "xmark").font(.system(size: 10, weight: .bold)).foregroundStyle(color)
            } else {
                ForEach(1...5, id: \.self) { bar in
                    Rectangle()
                        .fill(bar <= bars ? color : Theme.border)
                        .frame(width: 3, height: CGFloat(2 + bar * 2))
                }
            }
            if let status, status.failed == nil {
                Text("\(status.ms) ms").font(Theme.mono(9)).foregroundStyle(Theme.textFaint).padding(.leading, 4)
            }
        }
        .frame(height: 12)
    }
}

private struct AddServerSheet: View {
    let add: (String, String) -> Void
    @State private var name = "Minecraft Server"
    @State private var address = ""
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                HStack {
                    Text("Add Server").font(Theme.pixel(24)).foregroundStyle(Theme.text)
                    Spacer()
                    Button("Cancel") { dismiss() }.font(Theme.sans(14)).foregroundStyle(Theme.textMuted)
                }
                Field(title: "SERVER NAME", placeholder: "Minecraft Server", text: $name, keyboard: .default)
                Field(title: "SERVER ADDRESS", placeholder: "play.example.com", text: $address)
                PrimaryButton(title: "Done", icon: "checkmark") {
                    add(name, address)
                }
                .disabled(address.trimmingCharacters(in: .whitespaces).isEmpty)
            }
            .padding(20)
        }
        .background(Theme.background.ignoresSafeArea())
        .presentationDetents([.medium])

    }
}
