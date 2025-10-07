import SwiftUI

struct RelayItemView: View {
    let label: String
    let isSelected: Bool
    let isConnected: Bool
    let position: ItemPosition
    let level: Int

    var showSubtitle: Bool {
        !isSelected && isConnected
    }

    @Environment(\.isEnabled) var isEnabled

    var body: some View {
        HStack {
            if !isEnabled {
                Image.mullvadRedDot
            } else if isSelected {
                Image.mullvadIconTick
                    .foregroundStyle(Color.mullvadSuccessColor)
            }
            VStack(alignment: .leading) {
                Text(label)
                    .font(.mullvadSmallSemiBold)
                    .foregroundStyle(
                        isEnabled
                            ? isSelected
                                ? Color.mullvadSuccessColor
                                : Color.mullvadTextPrimary
                            : Color.mullvadTextPrimaryDisabled
                    )
                if showSubtitle {
                    Text("Connected server")
                        .font(.mullvadMiniSemiBold)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                }
            }
            Spacer()
        }
        .padding(.vertical, showSubtitle ? 8 : 16)
        .padding(.horizontal, CGFloat(16 * (level + 1)))
        .background {
            let backgroundColor = Color.colorForLevel(level)
            let corners: UIRectCorner =
                if level == 0 {
                    .allCorners
                } else {
                    switch position {
                    case .only: .allCorners
                    case .first: []
                    case .middle: []
                    case .last: [.bottomLeft, .bottomRight]
                    }
                }
            MullvadRoundedCorner(
                cornerRadius: 16,
                corners: corners
            )
            .foregroundStyle(backgroundColor)
        }
    }
}

#Preview {
    RelayItemView(
        label: "Test",
        isSelected: true,
        isConnected: false,
        position: .only,
        level: 0
    )
}
