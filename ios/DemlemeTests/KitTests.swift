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
}
