import SwiftUI

struct RelayItemView: View {
    let location: LocationNode
    let multihopContext: MultihopContext
    let level: Int
    let onSelect: () -> Void

    var disabled: Bool {
        !location.isActive || location.isExcluded
    }

    var subtitle: LocalizedStringKey? {
        if location.isConnected && !location.isSelected {
            return "Connected server"
        }
        return nil
    }

    var title: String {
        if location.isExcluded {
            switch multihopContext {
            case .entry:
                return """
                    \(location.name) (\(String(localized: 
                    String
                    .LocalizationValue(MultihopContext.exit.description))))
                    """
            case .exit:
                return """
                    \(location.name) (\(String(localized: 
                    String
                    .LocalizationValue(MultihopContext.entry.description))))
                    """
            }
        }
        return "\(location.name)"
    }

    var body: some View {
        Button {
            onSelect()
        } label: {
            HStack {
                if !location.isActive {
                    Image.mullvadRedDot
                } else if location.isSelected {
                    Image.mullvadIconTick
                        .foregroundStyle(Color.mullvadSuccessColor)
                }
                VStack(alignment: .leading) {
                    Text(title)
                        .font(.mullvadSmallSemiBold)
                        .foregroundStyle(
                            disabled
                                ? Color.mullvadTextPrimaryDisabled
                                : location.isSelected
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
                Color.colorForLevel(level)
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
        level: 0,
        onSelect: {}
    )
}
