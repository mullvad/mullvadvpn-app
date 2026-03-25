import SwiftUI

struct PressedExposingButton<Content: View>: View {
    let action: () -> Void
    let label: () -> Content
    let onPressedChange: ((Bool) -> Void)?
    let disabled: Bool

    struct MyButtonStyle: ButtonStyle {
        let action: () -> Void
        let label: () -> Content
        let onPressedChange: ((Bool) -> Void)?
        let disabled: Bool

        func makeBody(configuration: Configuration) -> some View {
            configuration.label
                .onChange(of: configuration.isPressed) {
                    if !disabled {
                        onPressedChange?(configuration.isPressed)
                    }
                }
                .opacity(configuration.isPressed && !disabled ? 0.6 : 1.0)
        }
    }

    var body: some View {
        Button(action: action, label: label)
            .buttonStyle(
                MyButtonStyle(
                    action: action,
                    label: label,
                    onPressedChange: onPressedChange,
                    disabled: disabled
                )
            )
    }
}
