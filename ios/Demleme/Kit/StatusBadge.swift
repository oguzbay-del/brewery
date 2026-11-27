import SwiftUI

/// A status tone. It maps to a token color.
enum GristoraTone: CaseIterable {
    case ok, warning, error, info, neutral

    var color: Color {
        switch self {
        case .ok: DemlemeColor.green
        case .warning: DemlemeColor.yellow
        case .error: DemlemeColor.red
        case .info: DemlemeColor.blue
        case .neutral: DemlemeColor.label2
        }
    }
}

/// A status with an icon and a word. Color is never the only signal.
struct StatusBadge: View {
    let tone: GristoraTone
    let label: String
    let systemImage: String

    /// Color for a tone. Internal so tests can check it.
    static func color(for tone: GristoraTone) -> Color { tone.color }

    var body: some View {
        HStack(spacing: DemlemeMetric.space1) {
            Image(systemName: systemImage)
                .imageScale(.small)
                .accessibilityHidden(true)
            Text(label)
                .font(.footnote.weight(.semibold))
                .fixedSize(horizontal: false, vertical: true)
        }
        .foregroundStyle(Self.color(for: tone))
        .padding(.horizontal, DemlemeMetric.space2)
        .padding(.vertical, DemlemeMetric.space1)
        .background(DemlemeColor.fill, in: Capsule())
        .accessibilityElement(children: .combine)
    }
}

#Preview("StatusBadge") {
    VStack(alignment: .leading, spacing: DemlemeMetric.space2) {
        StatusBadge(tone: .ok, label: "OK", systemImage: "checkmark.circle.fill")
        StatusBadge(tone: .warning, label: "Warning", systemImage: "exclamationmark.triangle.fill")
        StatusBadge(tone: .error, label: "Error", systemImage: "xmark.octagon.fill")
        StatusBadge(tone: .info, label: "Info", systemImage: "info.circle.fill")
        StatusBadge(tone: .neutral, label: "Neutral", systemImage: "circle")
    }
    .padding()
    .background(DemlemeColor.bg)
}
