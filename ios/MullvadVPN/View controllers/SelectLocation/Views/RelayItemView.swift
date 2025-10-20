import SwiftUI

struct RelayItemView: View {
    let location: LocationNode
    let multihopContext: MultihopContext
    let position: ItemPosition
    let level: Int
    let onSelect: () -> Void

    var isSelected: Bool {
        switch multihopContext {
        case .entry:
            location.isSelected == .entry
        case .exit:
            location.isSelected == .exit
        }
    }

    var isExcluded: Bool {
        switch multihopContext {
        case .entry:
            location.isExcludedFrom == .entry
        case .exit:
            location.isExcludedFrom == .exit
        }
    }

    var isConnected: Bool {
        switch multihopContext {
        case .entry:
            location.isConnected == .entry
        case .exit:
            location.isConnected == .exit
        }
    }

    var disabled: Bool {
        !location.isActive || isExcluded
    }

    var subtitle: LocalizedStringKey? {
        if isConnected && !isSelected {
            return "Connected server"
        }
        if isExcluded {
            switch multihopContext {
            case .entry:
                return "Selected as exit"
            case .exit:
                return "Selected as entry"
            }
        }
        return nil
    }

    var body: some View {
        Button {
            onSelect()
        } label: {
            HStack {
                if !location.isActive {
                    Image.mullvadRedDot
                } else if isSelected {
                    Image.mullvadIconTick
                        .foregroundStyle(Color.mullvadSuccessColor)
                }
                VStack(alignment: .leading) {
                    Text(location.name)
                        .font(.mullvadSmallSemiBold)
                        .foregroundStyle(
                            disabled
                                ? Color.mullvadTextPrimaryDisabled
                                : isSelected
                                    ? Color.mullvadSuccessColor
                                    : Color.mullvadTextPrimary
                        )
                    if let subtitle {
                        Text(subtitle)
                            .font(.mullvadMiniSemiBold)
                            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                    }
                }
                Spacer()
            }
            .padding(.vertical, subtitle != nil ? 8 : 16)
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
        .disabled(disabled)
    }
}

#Preview {
    RelayItemView(
        location: LocationNode(
            name: "A great location",
            code: "a-great-location"
        ),
        multihopContext: .exit,
        position: .only,
        level: 0,
        onSelect: {}
    )
}
