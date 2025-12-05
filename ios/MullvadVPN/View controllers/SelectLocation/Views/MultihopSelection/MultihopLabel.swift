import SwiftUI

struct MultihopLabel: View {
    let label: LocalizedStringKey
    let image: Image
    let onIconPositionChange: (CGRect) -> Void

    var body: some View {
        HStack(spacing: 10) {
            image
                .capturePosition(in: .multihopSelection) {
                    onIconPositionChange($0)
                }
            Text(label)
        }
        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
        .font(.mullvadMiniSemiBold)
    }
}
