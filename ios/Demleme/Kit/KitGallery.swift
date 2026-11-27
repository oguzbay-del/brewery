import SwiftUI

/// Shows each Kit view in normal, disabled and long-text cases. For previews and manual checks.
/// Sample text here is for design review only. It is not shipped in a screen.
struct KitGallery: View {
    private let longText = "A very long text that must wrap to a second line and then truncate with an ellipsis when it is still too long"

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: DemlemeMetric.space6) {
                buttons
                badges
                alarms
                summaries
                callouts
            }
            .padding(DemlemeMetric.space4)
        }
        .background(DemlemeColor.bg)
    }

    private var buttons: some View {
        section("Buttons") {
            Button("Primary") {}.buttonStyle(.gristoraPrimary)
            Button("Secondary") {}.buttonStyle(.gristoraSecondary)
            Button("Destructive") {}.buttonStyle(.gristoraDestructive)
            Button("Disabled primary") {}.buttonStyle(.gristoraPrimary).disabled(true)
            Button("Disabled secondary") {}.buttonStyle(.gristoraSecondary).disabled(true)
            Button(longText) {}.buttonStyle(.gristoraPrimary)
        }
    }

    private var badges: some View {
        section("StatusBadge") {
            ForEach(GristoraTone.allCases, id: \.self) { tone in
                StatusBadge(tone: tone, label: "\(tone)", systemImage: "circle.fill")
            }
            StatusBadge(tone: .warning, label: longText, systemImage: "exclamationmark.triangle.fill")
        }
    }

    private var alarms: some View {
        section("AlarmMark") {
            HStack(spacing: DemlemeMetric.space4) {
                ForEach(1 ... 3, id: \.self) { level in
                    AlarmMark(priority: level, accessibilityText: "Priority \(level)")
                }
            }
        }
    }

    private var summaries: some View {
        section("SummaryCard") {
            SummaryCard(title: "Gravity", value: "1.052", unit: "SG", footnote: "±0.002")
            SummaryCard(
                title: longText,
                value: "1234567.89",
                unit: "L",
                footnote: longText,
                badge: StatusBadge(tone: .error, label: "Error", systemImage: "xmark.octagon.fill")
            )
        }
    }

    private var callouts: some View {
        section("Callout") {
            ForEach(CalloutTone.allCases, id: \.self) { tone in
                Callout(tone: tone, title: "\(tone)", message: "Message text.")
            }
            Callout(
                tone: .warning,
                title: longText,
                message: longText,
                action: CalloutAction(title: "Action") {}
            )
            Callout(tone: .info, title: "Disabled action", action: CalloutAction(title: "Action") {})
                .disabled(true)
        }
    }

    private func section(_ name: String, @ViewBuilder content: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: DemlemeMetric.space3) {
            Text(name).font(.headline).foregroundStyle(DemlemeColor.label)
            content()
        }
    }
}

#Preview("Gallery light") {
    KitGallery().preferredColorScheme(.light)
}

#Preview("Gallery dark") {
    KitGallery().preferredColorScheme(.dark)
}

#Preview("Gallery accessibility3") {
    KitGallery().dynamicTypeSize(.accessibility3)
}

// Increased contrast: `colorSchemeContrast` is read-only in the environment.
// Turn on Increase Contrast in the Xcode preview Environment Overrides
// to check the high contrast color variants.
