import SwiftUI
import UIKit

extension World {
    /// "Survival · 2 hours ago · 26.3", as Select World's second line says it.
    var subtitle: String {
        let ago = RelativeDateTimeFormatter().localizedString(for: lastPlayed, relativeTo: Date())
        return [mode.capitalized, ago, version].filter { !$0.isEmpty }.joined(separator: " · ")
    }
}

/// A world's picture, or a grass-green square with its initial, pixels kept sharp.
struct WorldIcon: View {
    let world: World
    let size: CGFloat

    var body: some View {
        Group {
            if let icon = world.icon {
                Image(uiImage: icon).resizable().interpolation(.none).scaledToFill()
            } else {
                ZStack {
                    LinearGradient(colors: [Color(hex: 0x5D9A3A), Color(hex: 0x3F6E28)], startPoint: .top,
                                   endPoint: .bottom)
                    Text(String(world.name.prefix(1)).uppercased())
                        .font(Theme.pixel(size * 0.45, .bold)).foregroundStyle(.white)
                        .shadow(color: .black.opacity(0.4), radius: 0, x: 1, y: 1)
                }
            }
        }
        .frame(width: size, height: size)
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .overlay(RoundedRectangle(cornerRadius: 6).strokeBorder(Theme.border))
    }
}

/// A native library with the same file operations as Select World, plus search/sort/view controls.
struct WorldsSection: View {
    @ObservedObject var link: GameLink
    @State private var query = ""
    @State private var sort = WorldSort.recent
    @State private var grid = true
    let newWorld: () -> Void
    @State private var renaming: World?
    @State private var newName = ""
    @State private var deleting: World?
    @State private var backedUp: String?

    enum WorldSort: String, CaseIterable {
        case recent = "Last played", name = "Name", mode = "Game mode"
    }
    private var worlds: [World] {
        link.worlds.filter { query.isEmpty || $0.name.localizedCaseInsensitiveContains(query) }.sorted {
            switch sort {
            case .recent: return $0.lastPlayed > $1.lastPlayed
            case .name: return $0.name.localizedStandardCompare($1.name) == .orderedAscending
            case .mode: return $0.mode == $1.mode ? $0.name < $1.name : $0.mode < $1.mode
            }
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 24) {
            HStack {
                Text("\(worlds.count) \(worlds.count == 1 ? "world" : "worlds")")
                    .font(.subheadline).foregroundStyle(.secondary)
                Spacer()
                Menu {
                    Picker("Sort by", selection: $sort) {
                        ForEach(WorldSort.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    Toggle("Grid view", isOn: $grid)
                } label: { Image(systemName: "line.3.horizontal.decrease").frame(minWidth: 44, minHeight: 44) }
                    .accessibilityLabel("World view options")
                Button(action: newWorld) { Label("New World", systemImage: "plus") }
                    .controlSize(.large).launcherPrimaryButton().disabled(!link.canList)
                    .accessibilityIdentifier("new-world-button")
            }
            if !link.canList && link.worlds.isEmpty {
                ProgressView("Loading worlds…").frame(maxWidth: .infinity).padding(40)
            } else if worlds.isEmpty {
                VStack(spacing: 16) {
                    Image(systemName: "globe.europe.africa").font(.system(size: 48, weight: .ultraLight))
                    Text(query.isEmpty ? "Your next adventure starts here" : "No matching worlds").font(.title3.bold())
                    if query.isEmpty { Text("Create a world with the settings you want.").foregroundStyle(.secondary) }
                }.frame(maxWidth: .infinity).padding(.vertical, 64)
            } else if grid {
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 260, maximum: 420), spacing: 22)], spacing: 24) {
                    ForEach(worlds) { world in worldCard(world) }
                }
            } else {
                VStack(spacing: 0) {
                    ForEach(worlds) { world in
                        HStack(spacing: 14) {
                            Button { Task { await link.play(world) } } label: {
                                HStack(spacing: 14) {
                                    WorldIcon(world: world, size: 58)
                                    VStack(alignment: .leading, spacing: 5) {
                                        Text(world.name).font(.headline).foregroundStyle(.primary)
                                        Text(world.subtitle).font(.caption).foregroundStyle(.secondary)
                                    }
                                    Spacer()
                                }.contentShape(Rectangle())
                            }.buttonStyle(.plain).disabled(link.launching != nil)
                            Menu { actions(world) } label: { Image(systemName: "ellipsis").frame(width: 44, height: 44) }
                                .accessibilityLabel("Options for \(world.name)")
                        }.padding(16)
                        if world.id != worlds.last?.id { Divider().padding(.leading, 88) }
                    }
                }.card(radius: 22)
            }
        }
        .searchable(text: $query, prompt: "Search worlds")
        .alert("Rename world", isPresented: Binding(get: { renaming != nil }, set: { if !$0 { renaming = nil } })) {
            TextField("World name", text: $newName)
            Button("Rename") { if let world = renaming { Task { await link.rename(world, to: newName) } } }
            Button("Cancel", role: .cancel) {}
        }
        .confirmationDialog("Delete \(deleting?.name ?? "this world")?",
            isPresented: Binding(get: { deleting != nil }, set: { if !$0 { deleting = nil } }), titleVisibility: .visible) {
            Button("Delete World", role: .destructive) { if let world = deleting { Task { await link.delete(world) } } }
        } message: { Text("This permanently removes the world. Make a backup first if you want to keep it.") }
        .alert("Backup saved", isPresented: Binding(get: { backedUp != nil }, set: { if !$0 { backedUp = nil } })) {
            Button("OK", role: .cancel) {}
        } message: { Text(backedUp ?? "") }
    }

    private func worldCard(_ world: World) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Button { Task { await link.play(world) } } label: {
                ZStack(alignment: .bottomTrailing) {
                    LauncherLandscape(world: world)
                    LinearGradient(colors: [.clear, .black.opacity(0.30)], startPoint: .center, endPoint: .bottom)
                    Image(systemName: "play.circle.fill").font(.system(size: 36)).foregroundStyle(.white).padding(16)
                }.frame(height: 180).clipped().contentShape(Rectangle())
            }.buttonStyle(.plain).disabled(link.launching != nil).accessibilityLabel("Play \(world.name)")
            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(world.name).font(.headline).lineLimit(2)
                        Text(world.lastPlayed, style: .relative).font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer(minLength: 0)
                    Menu { actions(world) } label: { Image(systemName: "ellipsis").frame(width: 40, height: 44) }
                        .accessibilityLabel("Options for \(world.name)")
                }
                HStack(spacing: 8) {
                    Label(world.mode.capitalized, systemImage: world.mode == "creative" ? "cube" : "leaf")
                    Spacer()
                    Text(world.version)
                }.font(.caption).foregroundStyle(.secondary)
            }.padding(16)
        }
        .background(Theme.surface).clipShape(RoundedRectangle(cornerRadius: 22))
        .contextMenu { actions(world) }
    }

    @ViewBuilder private func actions(_ world: World) -> some View {
        Button { Task { await link.play(world) } } label: { Label("Play", systemImage: "play.fill") }
        Button { newName = world.name; renaming = world } label: { Label("Rename", systemImage: "pencil") }
        Button {
            Task {
                if let bytes = await link.backup(world) {
                    backedUp = "\(world.name) · \(ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .file))"
                    Haptics.done()
                }
            }
        } label: { Label("Back Up", systemImage: "externaldrive.badge.plus") }
        Button(role: .destructive) { deleting = world } label: { Label("Delete", systemImage: "trash") }
    }
}

struct WorldCreationSettings: Codable {
    var name = "New World"
    var seed = ""
    var mode = "survival"
    var difficulty = "normal"
    var commands = false
    var flat = false
    var structures = true
    var bonus_chest = false
}

/// Every switch is backed by the game's CreateWorldScreen options, not a decorative preference.
struct NewWorldSheet: View {
    let create: (WorldCreationSettings) -> Void
    @State private var settings = WorldCreationSettings()
    @Environment(\.dismiss) private var dismiss
    private var hardcore: Bool { settings.mode == "hardcore" }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("World name", text: $settings.name)
                        .accessibilityIdentifier("creation-world-name")
                    Picker("Game mode", selection: $settings.mode) {
                        Label("Survival", systemImage: "leaf").tag("survival")
                        Label("Creative", systemImage: "cube").tag("creative")
                        Label("Hardcore", systemImage: "heart.slash").tag("hardcore")
                    }
                    .pickerStyle(.menu).accessibilityIdentifier("creation-mode")
                    Picker("Difficulty", selection: $settings.difficulty) {
                        ForEach(["peaceful", "easy", "normal", "hard"], id: \.self) { Text($0.capitalized).tag($0) }
                    }.pickerStyle(.menu).disabled(hardcore).accessibilityIdentifier("creation-difficulty")
                    Toggle("Allow Cheats", isOn: $settings.commands).disabled(hardcore).accessibilityIdentifier("creation-cheats")
                } header: { Text("Game") } footer: {
                    Text(hardcore ? "One life, locked to Hard difficulty. Cheats and the bonus chest are disabled." :
                         settings.mode == "creative" ? "Unlimited blocks, flying and instant building." : "Gather resources, craft and explore.")
                }
                Section("World generation") {
                    TextField("Seed (optional)", text: $settings.seed).textInputAutocapitalization(.never).autocorrectionDisabled()
                    Picker("World type", selection: $settings.flat) {
                        Text("Default").tag(false)
                        Text("Superflat").tag(true)
                    }
                    Toggle("Generate Structures", isOn: $settings.structures)
                    Toggle("Bonus Chest", isOn: $settings.bonus_chest).disabled(hardcore)
                }
            }
            .navigationTitle("New World")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Create") { create(settings) }
                        .disabled(settings.name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        .accessibilityIdentifier("creation-confirm")
                }
            }
            .onChange(of: settings.mode) { mode in
                settings.commands = mode == "creative"
                if mode == "hardcore" { settings.difficulty = "hard"; settings.bonus_chest = false }
            }
        }
        .presentationDetents([.large])
        .presentationDragIndicator(.visible)
    }
}

/// Artwork is the user's world thumbnail or imported title panorama, never bundled game assets.
struct LauncherLandscape: View {
    var index: Int = 0
    var world: World? = nil
    @State private var panorama: UIImage?
    var body: some View {
        GeometryReader { geometry in
            Group {
                if let image = world?.icon ?? panorama {
                    Image(uiImage: image).resizable().scaledToFill()
                } else {
                    // An original, quiet landscape while resources are not imported yet.
                    ZStack {
                        LinearGradient(colors: [Color(red: 0.18, green: 0.48, blue: 0.57),
                                                Color(red: 0.55, green: 0.71, blue: 0.68)],
                                       startPoint: .top, endPoint: .bottom)
                        Image(systemName: "mountain.2.fill").resizable().scaledToFit()
                            .foregroundStyle(.white.opacity(0.16)).padding(32).offset(y: 30)
                    }
                }
            }.frame(width: geometry.size.width, height: geometry.size.height).clipped()
        }
        .accessibilityHidden(true)
        .allowsHitTesting(false)
        .task {
            let url = GameFolder.resources.appendingPathComponent("assets/minecraft/textures/gui/title/background/panorama_\(index % 4).png")
            if let image = UIImage(contentsOfFile: url.path), image.size.width > 16, image.size.height > 16 {
                panorama = image
            }
        }
    }
}
