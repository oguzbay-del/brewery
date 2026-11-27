import SwiftUI

/// A card that shows one number. The value never truncates.
struct SummaryCard: View {
    let title: String
    let value: String
    let unit: String
    var footnote: String?
    var badge: StatusBadge?

    var body: some View {
        VStack(alignment: .leading, spacing: DemlemeMetric.space2) {
            HStack(alignment: .firstTextBaseline, spacing: DemlemeMetric.space2) {
                Text(title)
                    .font(.footnote.weight(.semibold))
                    .foregroundStyle(DemlemeColor.tint)
                    .fixedSize(horizontal: false, vertical: true)
                Spacer(minLength: 0)
                if let badge {
                    badge
                }
            }
            HStack(alignment: .firstTextBaseline, spacing: DemlemeMetric.space1) {
                Text(value)
                    .font(.system(.largeTitle, design: .rounded, weight: .semibold))
                    .monospacedDigit()
                    .foregroundStyle(DemlemeColor.label)
                    .lineLimit(1)
                    .minimumScaleFactor(0.8)
                Text(unit)
                    .font(.subheadline)
                    .foregroundStyle(DemlemeColor.label2)
            }
            if let footnote {
                Text(footnote)
                    .font(.footnote)
                    .foregroundStyle(DemlemeColor.label2)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(DemlemeMetric.space4)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(DemlemeColor.card, in: RoundedRectangle(cornerRadius: DemlemeMetric.radiusLg, style: .continuous))
        .accessibilityElement(children: .combine)
    }
}

#Preview("SummaryCard") {
    VStack(spacing: DemlemeMetric.space3) {
        SummaryCard(title: "Gravity", value: "1.052", unit: "SG", footnote: "±0.002")
        SummaryCard(
            title: "Temperature",
            value: "18.5",
            unit: "°C",
            badge: StatusBadge(tone: .ok, label: "In range", systemImage: "checkmark.circle.fill")
        )
    }
    .padding()
    .background(DemlemeColor.bg)
}
