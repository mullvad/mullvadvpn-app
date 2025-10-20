import SwiftUI

struct DaitaWarningView: View {
    let onOpenDaitaSettings: () -> Void
    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Text(
                """
                The entry server for \("multihop") is currently overridden by \("DAITA"). To select an entry server, \
                please first enable “\("Direct only")” or disable “\("DAITA")” in the settings.
                """
            )
            .multilineTextAlignment(.center)
            .foregroundStyle(Color.mullvadTextSecondary)
            .font(.mullvadSmall)
            MainButton(text: "Open \("DAITA") settings", style: .default) {
                onOpenDaitaSettings()
            }
            Spacer()
        }
        .padding()
    }
}

#Preview {
    DaitaWarningView(onOpenDaitaSettings: {})
        .background(Color.mullvadBackground)
}
