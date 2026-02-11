import SwiftUI

struct RelayItemView: View {
    let location: LocationNode
    let multihopContext: MultihopContext
    let level: Int
    let subtitle: LocalizedStringKey?
    var isLastInList = true
    let onSelect: () -> Void

    var disabled: Bool {
        !location.isActive || location.isExcluded
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
                locationStatusIndicator()
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
            .padding(.leading, CGFloat(16 * (level + 1)))
            .padding(.trailing, 16)
            .background {
                Color.colorForLevel(level)
            }
        }
        .disabled(disabled)
        .clipShape(
            UnevenRoundedRectangle(
                cornerRadii: .init(
                    topLeading: level == 0 ? 16 : 0,
                    bottomLeading: isLastInList ? 16 : 0,
                    bottomTrailing: isLastInList ? 16 : 0,
                    topTrailing: level == 0 ? 16 : 0
                )
            )
        )

    }

    @ViewBuilder
    func locationStatusIndicator() -> some View {
        Group {
            if !location.isActive {
                Image.mullvadRedDot
            } else if location.isSelected {
                Image.mullvadIconTick
                    .foregroundStyle(Color.mullvadSuccessColor)
            }
        }
        .frame(width: 24, height: 24)
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
        subtitle: nil,
        onSelect: {}
    )
}
