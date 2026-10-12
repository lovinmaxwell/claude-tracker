import SwiftUI
import ClaudeUsageCore

/// Tracks whether the pointer is over the widget; the AppKit side owns the truth.
final class HoverState: ObservableObject {
    @Published var expanded = false
}

enum WidgetPalette {
    static let session = Color(red: 0.788, green: 0.392, blue: 0.259)
    static let week = Color(red: 0.353, green: 0.478, blue: 0.416)
    static let hot = Color(red: 0.851, green: 0.294, blue: 0.204)
    static let watch = Color(red: 0.851, green: 0.631, blue: 0.231)

    static func color(for tone: Tone) -> Color {
        switch tone {
        case .ok: return week
        case .watch: return watch
        case .hot: return hot
        case .stale, .wait: return Color.secondary
        }
    }

    /// Rings keep a fixed colour per window so they stay distinguishable; only a hot ring turns red.
    static func ring(index: Int, tone: Tone) -> Color {
        if tone == .hot { return hot }
        return index == 0 ? session : week
    }
}

struct RingModel: Identifiable {
    let id: String
    let label: String
    let used: Double?
    let resetsAt: Date?
    let color: Color

    var fraction: Double { (used ?? 0) / 100 }
}

/// Everything the views need, derived once per tick from the store's state.
struct WidgetModel {
    let rings: [RingModel]
    let peak: RingModel?
    let status: UsageStatus
    let age: String?

    init(snapshot: UsageSnapshot?, lastError: UsageError?, now: Date) {
        let windows = snapshot?.windows ?? []
        rings = windows.enumerated().map { index, window in
            RingModel(
                id: window.id,
                label: window.label,
                used: window.used,
                resetsAt: window.resetsAt,
                color: WidgetPalette.ring(index: index, tone: UsageAnalysis.tone(forUsed: window.used))
            )
        }
        let peakWindow = UsageAnalysis.peak(windows)
        peak = rings.first { $0.id == peakWindow?.id }
        status = UsageAnalysis.summarize(
            snapshot: snapshot,
            isStale: snapshot != nil && lastError != nil,
            error: lastError,
            now: now
        )
        age = Formatters.age(of: snapshot?.fetchedAt, now: now)
    }
}

struct WidgetRootView: View {
    @ObservedObject var store: UsageStore
    @ObservedObject var hover: HoverState

    var body: some View {
        // Ticks every 15s so countdowns and "Updated Xs ago" stay current.
        TimelineView(.periodic(from: Date(), by: 15)) { context in
            WidgetContent(store: store, hover: hover, now: context.date)
        }
        .ignoresSafeArea()
    }
}

/// Both layers stacked; the hover state fades between them.
struct WidgetContent: View {
    @ObservedObject var store: UsageStore
    @ObservedObject var hover: HoverState
    let now: Date

    private var model: WidgetModel {
        WidgetModel(snapshot: store.snapshot, lastError: store.lastError, now: now)
    }

    var body: some View {
        // The base is Color.clear so the layout always takes exactly the window's
        // current size. The layers are overlaid at the top-left rather than stacked
        // in a ZStack: the hidden 336x204 card would otherwise inflate the layout,
        // and macOS would centre that in the small resting window, pushing the
        // rings out of view.
        Color.clear
            .overlay(alignment: .topLeading) {
                ZStack(alignment: .topLeading) {
                    RingsOnlyView(model: model)
                        .frame(width: WidgetLayout.rest.width, height: WidgetLayout.rest.height)
                        .opacity(hover.expanded ? 0 : 1)
                    CardView(model: model, now: now)
                        .frame(width: WidgetLayout.card.width, height: WidgetLayout.card.height)
                        .opacity(hover.expanded ? 1 : 0)
                }
                .animation(.easeInOut(duration: WidgetLayout.animationDuration), value: hover.expanded)
            }
    }
}

/// Resting state: only the rings, with nothing behind them.
struct RingsOnlyView: View {
    let model: WidgetModel

    var body: some View {
        RingStack(rings: model.rings, size: 96, stroke: 9, gap: 3, glyphSize: 16, trackOpacity: 0.28)
            // A soft shadow keeps the rings readable on light and dark wallpapers alike.
            .shadow(color: Color.black.opacity(0.4), radius: 3, x: 0, y: 1)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

/// Hovered state: the full picture.
struct CardView: View {
    let model: WidgetModel
    let now: Date

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            StatusChip(status: model.status)
            Text(model.status.detail ?? " ")
                .font(.system(size: 12))
                .foregroundColor(.secondary)
                .lineLimit(2)
                .frame(height: 32, alignment: .topLeading)
                .padding(.top, 4)

            HStack(spacing: 16) {
                RingStack(rings: model.rings, size: 96, stroke: 9, gap: 3, glyphSize: 16)
                VStack(alignment: .leading, spacing: 12) {
                    ForEach(model.rings) { ring in
                        RingRow(ring: ring, now: now)
                    }
                    if model.rings.isEmpty {
                        Text("No usage reported yet.")
                            .font(.system(size: 11.5))
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(maxHeight: .infinity)

            HStack {
                Text("Double-click opens Claude Code")
                Spacer()
                Text(model.age.map { "Updated \($0)" } ?? "")
                    .monospacedDigit()
            }
            .font(.system(size: 11))
            .foregroundColor(.secondary)
        }
        .padding(.top, 14)
        .padding(.bottom, 12)
        .padding(.horizontal, 16)
    }
}

struct StatusChip: View {
    let status: UsageStatus

    var body: some View {
        let color = WidgetPalette.color(for: status.tone)
        HStack(spacing: 6) {
            Circle().fill(color).frame(width: 7, height: 7)
            Text(status.headline)
                .font(.system(size: 12, weight: .semibold))
        }
        .padding(.horizontal, 9)
        .frame(height: 22)
        .foregroundColor(color)
        .background(Capsule().fill(color.opacity(0.14)))
    }
}

struct RingRow: View {
    let ring: RingModel
    let now: Date

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 7) {
                RoundedRectangle(cornerRadius: 3).fill(ring.color).frame(width: 8, height: 8)
                Text(ring.label)
                    .font(.system(size: 12.5, weight: .medium))
                    .foregroundColor(.secondary)
                Spacer(minLength: 4)
                Text(Formatters.percent(ring.used))
                    .font(.system(size: 15, weight: .bold))
                    .monospacedDigit()
                    .foregroundColor(UsageAnalysis.tone(forUsed: ring.used) == .hot ? WidgetPalette.hot : .primary)
            }
            Text(resetLine)
                .font(.system(size: 11))
                .foregroundColor(.secondary)
                .lineLimit(1)
                .padding(.leading, 15)
        }
    }

    private var resetLine: String {
        guard
            let countdown = Formatters.countdown(to: ring.resetsAt, now: now),
            let clock = Formatters.resetClock(ring.resetsAt, now: now)
        else { return "No reset time reported" }
        return "Resets in \(countdown) · \(clock)"
    }
}

/// Concentric progress rings, outermost first, with a spark in the middle.
struct RingStack: View {
    let rings: [RingModel]
    let size: CGFloat
    let stroke: CGFloat
    let gap: CGFloat
    let glyphSize: CGFloat
    /// How visible the empty part of each ring is; the resting rings need more.
    var trackOpacity: Double = 0.1

    var body: some View {
        ZStack {
            ForEach(Array(rings.enumerated()), id: \.element.id) { index, ring in
                let diameter = size - stroke - CGFloat(index) * 2 * (stroke + gap)
                Circle()
                    .stroke(Color.primary.opacity(trackOpacity), lineWidth: stroke)
                    .frame(width: diameter, height: diameter)
                if ring.used != nil {
                    Circle()
                        .trim(from: 0, to: max(ring.fraction, ring.fraction > 0 ? 0.015 : 0))
                        .stroke(ring.color, style: StrokeStyle(lineWidth: stroke, lineCap: .round))
                        .rotationEffect(.degrees(-90))
                        .frame(width: diameter, height: diameter)
                        .animation(.easeOut(duration: 0.9), value: ring.fraction)
                }
            }
            if rings.isEmpty {
                Circle()
                    .stroke(Color.primary.opacity(trackOpacity), lineWidth: stroke)
                    .frame(width: size - stroke, height: size - stroke)
            }
            Image(systemName: "sparkle")
                .font(.system(size: glyphSize, weight: .semibold))
                .foregroundColor(.secondary)
        }
        .frame(width: size, height: size)
    }
}
