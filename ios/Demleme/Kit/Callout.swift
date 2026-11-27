import SwiftUI

/// Tone of a callout.
enum CalloutTone: CaseIterable {
    case info, success, warning, error

    var color: Color {
        switch self {
        case .info: DemlemeColor.blue
        case .success: DemlemeColor.green
        case .warning: DemlemeColor.yellow
        case .error: DemlemeColor.red
        }
    }

    var systemImage: String {
        switch self {
        case .info: "info.circle.fill"
        case .success: "checkmark.circle.fill"
        case .warning: "exclamationmark.triangle.fill"
        case .error: "xmark.octagon.fill"
        }
    }
}

/// An optional action of a callout. The caller gives the title.
struct CalloutAction {
    let title: String
    let handler: () -> Void
}

/// A short status note on a screen. Color is on the icon only.
struct Callout: View {
    let tone: CalloutTone
    let title: String
    var message: String?
    var action: CalloutAction?

    /// Color for a tone. Internal so tests can check it.
    static func color(for tone: CalloutTone) -> Color { tone.color }

    var body: some View {
        HStack(alignment: .top, spacing: DemlemeMetric.space3) {
            Image(systemName: tone.systemImage)
                .foregroundStyle(Self.color(for: tone))
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: DemlemeMetric.space2) {
                VStack(alignment: .leading, spacing: DemlemeMetric.space1) {
                    Text(title)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(DemlemeColor.label)
                    if let message {
                        Text(message)
                            .font(.footnote)
                            .foregroundStyle(DemlemeColor.label)
                    }
                }
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityElement(children: .combine)
                if let action {
                    Button(action.title, action: action.handler)
                        .buttonStyle(.gristoraSecondary)
                }
            }
            Spacer(minLength: 0)
        }
        .padding(DemlemeMetric.space3)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(DemlemeColor.card, in: RoundedRectangle(cornerRadius: DemlemeMetric.radiusMd, style: .continuous))
    }
}

#Preview("Callout") {
    VStack(spacing: DemlemeMetric.space3) {
        ForEach(CalloutTone.allCases, id: \.self) { tone in
            Callout(tone: tone, title: "Title", message: "Message text.")
        }
        Callout(tone: .error, title: "With action", action: CalloutAction(title: "Retry") {})
    }
    .padding()
    .background(DemlemeColor.bg)
}
