import SwiftUI

struct MultihopWhenNeededInfoView: View {
    let onSetMultihopToAlways: () -> Void
    var body: some View {
        VStack(spacing: 16) {
            Spacer()

            Image.mullvadIconMultihopWhenNeeded
                .resizable()
                .frame(width: 48, height: 48)

            Group {
                Text(
                    "The entry server is currently selected automatically to ensure your current settings "
                        + "work with your selected location."
                )
                Text(
                    "To manually select an entry server, please switch multihop mode to “\("Always")”."
                )
            }
            .multilineTextAlignment(.center)
            .foregroundStyle(Color.mullvadTextSecondary)
            .font(.mullvadSmall)

            Spacer()

            MainButton(text: "Set multihop to “\("Always")“", style: .default) {
                onSetMultihopToAlways()
            }
        }
        .padding()
    }
}

#Preview {
    MultihopWhenNeededInfoView(onSetMultihopToAlways: {})
        .background(Color.mullvadBackground)
}
