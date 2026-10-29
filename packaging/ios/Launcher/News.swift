import SwiftUI

/// One change to the iPhone port: a commit on the repository's main branch, its first line the headline and
/// the rest the story, as the commits here are written.
struct Change: Identifiable, Equatable {
    let id: String
    let title: String
    let body: String
    let date: Date
    let url: URL?
}

/// What's New: the port's latest changes from GitHub, kept for the next time there is no network.
@MainActor
final class Changelog: ObservableObject {
    @Published private(set) var changes: [Change] = []
    @Published private(set) var failed: String?
    @Published private(set) var loading = false

    static let repository = "heyskylo/horizon-ios"
    private static let api = URL(string: "https://api.github.com/repos/\(repository)/commits?sha=main&per_page=40")!
    private static var cache: URL {
        FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("horizon-changelog.json")
    }

    init() {
        if let data = try? Data(contentsOf: Self.cache) {
            changes = Self.parse(data)
        }
    }

    func load() async {
        loading = true
        defer { loading = false }
        var request = URLRequest(url: Self.api)
        request.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0
            guard status == 200 else {
                // 403 is GitHub's hourly limit for requests without an account.
                failed = status == 403 ? "GitHub asked to wait a while; showing the last list." : "GitHub answered \(status)."
                return
            }
            let parsed = Self.parse(data)
            if !parsed.isEmpty {
                changes = parsed
                failed = nil
                try? data.write(to: Self.cache)
            }
        } catch {
            failed = changes.isEmpty ? error.localizedDescription : "Offline; showing the last list."
        }
    }

    private static func parse(_ data: Data) -> [Change] {
        guard let list = (try? JSONSerialization.jsonObject(with: data)) as? [[String: Any]] else { return [] }
        let dates = ISO8601DateFormatter()
        return list.compactMap { item in
            guard let sha = item["sha"] as? String,
                  let commit = item["commit"] as? [String: Any],
                  let message = commit["message"] as? String
            else { return nil }
            let when = ((commit["committer"] as? [String: Any])?["date"] as? String).flatMap(dates.date(from:))
            let lines = message.components(separatedBy: "\n")
            let title = lines.first ?? ""
            // The story without its attribution lines.
            let body = lines.dropFirst()
                .filter { !$0.lowercased().hasPrefix("co-authored-by:") && !$0.hasPrefix("Signed-off-by:") }
                .joined(separator: "\n")
                .trimmingCharacters(in: .whitespacesAndNewlines)
                // Paragraphs are hard-wrapped at 110 columns: join the lines of each one again.
                .components(separatedBy: "\n\n")
                .map { $0.replacingOccurrences(of: "\n", with: " ") }
                .joined(separator: "\n\n")
            // Merges say nothing a player reads.
            if title.hasPrefix("Merge ") { return nil }
            return Change(id: sha, title: title, body: body, date: when ?? .distantPast,
                          url: (item["html_url"] as? String).flatMap(URL.init(string:)))
        }
    }
}

struct NewsPage: View {
    @StateObject private var log = Changelog()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            PageTitle(title: "What's New", subtitle: "The latest changes to Horizon on iPhone.")
            if let failed = log.failed {
                Label(failed, systemImage: "wifi.exclamationmark")
                    .font(Theme.sans(12)).foregroundStyle(Theme.warn)
            }
            if log.changes.isEmpty {
                if log.loading {
                    ProgressView().tint(Theme.accentText).frame(maxWidth: .infinity).padding(30)
                } else {
                    EmptyNote(icon: "sparkles", text: "Nothing to show yet.")
                    SecondaryButton(title: "Try again", icon: "arrow.clockwise") {
                        Task { await log.load() }
                    }
                }
            }
            ForEach(days) { day in
                Text(day.date.formatted(date: .complete, time: .omitted).uppercased())
                    .font(Theme.mono(9)).foregroundStyle(Theme.textMuted)
                    .padding(.top, 6)
                VStack(spacing: 0) {
                    ForEach(day.changes) { change in
                        ChangeRow(change: change)
                        if change != day.changes.last {
                            Divider().overlay(Theme.border).padding(.leading, 14)
                        }
                    }
                }
                .card()
            }
            Link(destination: URL(string: "https://github.com/\(Changelog.repository)/commits/main")!) {
                Label("Every change on GitHub", systemImage: "arrow.up.right.square")
                    .font(Theme.sans(13, .semiBold)).foregroundStyle(Theme.accentText)
            }
            Link(destination: URL(string: "https://github.com/ludovic111/horizon/releases")!) {
                Label("Horizon's desktop releases", systemImage: "arrow.up.right.square")
                    .font(Theme.sans(13, .semiBold)).foregroundStyle(Theme.accentText)
            }
        }
        .task { await log.load() }
    }

    /// The changes by the day they were made, newest first.
    private var days: [Day] {
        let calendar = Calendar.current
        let grouped = Dictionary(grouping: log.changes) { calendar.startOfDay(for: $0.date) }
        return grouped.keys.sorted(by: >).map { Day(date: $0, changes: grouped[$0] ?? []) }
    }

    private struct Day: Identifiable {
        let date: Date
        let changes: [Change]
        var id: Date { date }
    }
}

private struct ChangeRow: View {
    let change: Change
    @State private var open = false

    var body: some View {
        Button {
            Haptics.tap()
            withAnimation(.easeInOut(duration: 0.2)) { open.toggle() }
        } label: {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .top, spacing: 10) {
                    Rectangle().fill(Theme.accentBright).frame(width: 5, height: 5).padding(.top, 7)
                    Text(change.title).font(Theme.sans(14, .semiBold)).foregroundStyle(Theme.text)
                        .multilineTextAlignment(.leading)
                    Spacer(minLength: 0)
                    if !change.body.isEmpty {
                        Image(systemName: "chevron.down")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundStyle(Theme.textFaint)
                            .rotationEffect(.degrees(open ? 180 : 0))
                            .padding(.top, 3)
                    }
                }
                if open, !change.body.isEmpty {
                    Text(change.body).font(Theme.sans(12)).foregroundStyle(Theme.textMuted)
                        .multilineTextAlignment(.leading)
                        .fixedSize(horizontal: false, vertical: true)
                        .padding(.leading, 15)
                }
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
