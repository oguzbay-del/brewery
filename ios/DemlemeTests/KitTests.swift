@testable import Demleme
import SwiftUI
import Testing

@MainActor
struct KitTests {
    @Test func statusToneMapsToColor() {
        #expect(StatusBadge.color(for: .ok) == DemlemeColor.green)
        #expect(StatusBadge.color(for: .warning) == DemlemeColor.yellow)
        #expect(StatusBadge.color(for: .error) == DemlemeColor.red)
        #expect(StatusBadge.color(for: .info) == DemlemeColor.blue)
        #expect(StatusBadge.color(for: .neutral) == DemlemeColor.label2)
    }

    @Test func calloutToneMapsToColor() {
        #expect(Callout.color(for: .info) == DemlemeColor.blue)
        #expect(Callout.color(for: .success) == DemlemeColor.green)
        #expect(Callout.color(for: .warning) == DemlemeColor.yellow)
        #expect(Callout.color(for: .error) == DemlemeColor.red)
    }

    @Test func alarmPriorityMapsToColor() {
        #expect(AlarmMark.fillColor(for: 1) == DemlemeColor.hpA1)
        #expect(AlarmMark.fillColor(for: 2) == DemlemeColor.hpA2)
        #expect(AlarmMark.fillColor(for: 3) == DemlemeColor.hpA3)
        #expect(AlarmMark.inkColor(for: 2) == DemlemeColor.hpA2Ink)
        #expect(AlarmMark.clamped(0) == 1)
        #expect(AlarmMark.clamped(9) == 3)
    }

    @Test func viewsCanBeMade() {
        let badge = StatusBadge(tone: .ok, label: "OK", systemImage: "checkmark")
        _ = badge.body
        _ = AlarmMark(priority: 2, accessibilityText: "Priority 2").body
        _ = SummaryCard(title: "T", value: "1", unit: "u", footnote: "f", badge: badge).body
        _ = Callout(tone: .info, title: "T", message: "M", action: CalloutAction(title: "A") {}).body
        _ = KitGallery().body
        _ = Button("B") {}.buttonStyle(.gristoraPrimary)
        _ = Button("B") {}.buttonStyle(.gristoraSecondary)
        _ = Button("B") {}.buttonStyle(.gristoraDestructive)
    }
}
