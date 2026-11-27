import SwiftUI

/// Shared look of the Kit buttons. The variant sets fill and text color.
private struct GristoraButtonBody: View {
    let configuration: ButtonStyleConfiguration
    let fill: Color
    let ink: Color

    @Environment(\.isEnabled) private var isEnabled
    @ScaledMetric(relativeTo: .body) private var minHeight = DemlemeMetric.controlHeight

    var body: some View {
        configuration.label
            .font(.body.weight(.semibold))
            .multilineTextAlignment(.center)
            .foregroundStyle(isEnabled ? ink : DemlemeColor.label3)
            .padding(.horizontal, DemlemeMetric.space4)
            .padding(.vertical, DemlemeMetric.space2)
            .frame(minHeight: minHeight)
            .background(fill, in: RoundedRectangle(cornerRadius: DemlemeMetric.radiusMd, style: .continuous))
            .opacity(configuration.isPressed ? 0.7 : 1)
            .contentShape(RoundedRectangle(cornerRadius: DemlemeMetric.radiusMd, style: .continuous))
    }
}

/// The one main action of a screen.
struct GristoraPrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        GristoraButtonBody(configuration: configuration, fill: DemlemeColor.tintFill, ink: DemlemeColor.tintInk)
    }
}

/// A second action next to the main action.
struct GristoraSecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        GristoraButtonBody(configuration: configuration, fill: DemlemeColor.fill, ink: DemlemeColor.tint)
    }
}

/// An action that deletes or cannot be undone. The caller must ask for confirmation.
struct GristoraDestructiveButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        GristoraButtonBody(configuration: configuration, fill: DemlemeColor.fill, ink: DemlemeColor.red)
    }
}

extension ButtonStyle where Self == GristoraPrimaryButtonStyle {
    static var gristoraPrimary: GristoraPrimaryButtonStyle { GristoraPrimaryButtonStyle() }
}

extension ButtonStyle where Self == GristoraSecondaryButtonStyle {
    static var gristoraSecondary: GristoraSecondaryButtonStyle { GristoraSecondaryButtonStyle() }
}

extension ButtonStyle where Self == GristoraDestructiveButtonStyle {
    static var gristoraDestructive: GristoraDestructiveButtonStyle { GristoraDestructiveButtonStyle() }
}

#Preview("Buttons") {
    VStack(spacing: DemlemeMetric.space3) {
        Button("Primary") {}.buttonStyle(.gristoraPrimary)
        Button("Secondary") {}.buttonStyle(.gristoraSecondary)
        Button("Destructive") {}.buttonStyle(.gristoraDestructive)
        Button("Disabled") {}.buttonStyle(.gristoraPrimary).disabled(true)
    }
    .padding()
    .background(DemlemeColor.bg)
}
