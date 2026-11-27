import SwiftUI

/// ISA-101 alarm mark. Priority 1 is a square, 2 a diamond, 3 a circle.
/// The number is inside. The caller gives the accessibility label.
struct AlarmMark: View {
    let priority: Int
    let accessibilityText: String

    @ScaledMetric(relativeTo: .body) private var size = DemlemeMetric.badgeSize

    /// Priority clamped to 1...3.
    static func clamped(_ priority: Int) -> Int { min(max(priority, 1), 3) }

    static func fillColor(for priority: Int) -> Color {
        switch clamped(priority) {
        case 1: DemlemeColor.hpA1
        case 2: DemlemeColor.hpA2
        default: DemlemeColor.hpA3
        }
    }

    static func inkColor(for priority: Int) -> Color {
        clamped(priority) == 2 ? DemlemeColor.hpA2Ink : Color.white
    }

    var body: some View {
        let level = Self.clamped(priority)
        ZStack {
            shape(level)
            Text("\(level)")
                .font(.system(.footnote, design: .rounded, weight: .bold))
                .monospacedDigit()
                .foregroundStyle(Self.inkColor(for: level))
        }
        .frame(width: size, height: size)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(accessibilityText)
    }

    @ViewBuilder
    private func shape(_ level: Int) -> some View {
        let fill = Self.fillColor(for: level)
        switch level {
        case 1:
            Rectangle().fill(fill)
        case 2:
            AlarmDiamond().fill(fill)
        default:
            Circle().fill(fill)
        }
    }
}

private struct AlarmDiamond: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: rect.midX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.midY))
        path.addLine(to: CGPoint(x: rect.midX, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.midY))
        path.closeSubpath()
        return path
    }
}

#Preview("AlarmMark") {
    HStack(spacing: DemlemeMetric.space4) {
        AlarmMark(priority: 1, accessibilityText: "Priority 1")
        AlarmMark(priority: 2, accessibilityText: "Priority 2")
        AlarmMark(priority: 3, accessibilityText: "Priority 3")
    }
    .padding()
    .background(DemlemeColor.hpBg)
}
