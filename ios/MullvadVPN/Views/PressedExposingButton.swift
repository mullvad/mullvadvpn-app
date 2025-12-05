import SwiftUI

struct PressedExposingButton<Content: View>: View {
    let action: () -> Void
    let label: () -> Content
    let onPressedChange: ((Bool) -> Void)?
    struct MyButtonStyle: ButtonStyle {
        let action: () -> Void
        let label: () -> Content
        let onPressedChange: ((Bool) -> Void)?

        func makeBody(configuration: Configuration) -> some View {
            configuration.label
                .onChange(of: configuration.isPressed) {
                    onPressedChange?(configuration.isPressed)
                }
                .opacity(configuration.isPressed ? 0.6 : 1.0)
        }
    }

    var body: some View {
        Button(action: action, label: label)
            .buttonStyle(
                MyButtonStyle(
                    action: action,
                    label: label,
                    onPressedChange: onPressedChange
                )
            )
    }
}
