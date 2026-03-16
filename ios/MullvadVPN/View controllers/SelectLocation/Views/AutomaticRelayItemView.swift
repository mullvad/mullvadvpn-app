import SwiftUI

struct AutomaticRelayItemView: View {
    let location: LocationNode
    var subtitle: LocalizedStringKey?
    let onSelectLocation: (LocationNode) -> Void
    let onSelectInfo: () -> Void

    var body: some View {
        HStack(spacing: 2) {
            SegmentedListItemOLD(
                title: LocalizedStringKey(location.name),
                subtitle: subtitle,
                onSelect: { onSelectLocation(location) },
                minHeight: UIMetrics.LocationList.cellMinHeight,
                leadingView: AnyView(locationStatusIndicator()),
                secondaryButtonImage: .mullvadIconInfo,
                onSecondarySelect: onSelectInfo

            )
            .padding(.horizontal, 16)
            //            .background {
            //                Color.colorForLevel(0)
            //            }
        }
    }

    @ViewBuilder
    func locationStatusIndicator() -> some View {
        Group {
            //            if !location.isActive {
            //                Image.mullvadRedDot
            //            } else if location.isSelected {
            //                Image.mullvadIconTick
            //                    .foregroundStyle(Color.mullvadSuccessColor)
            //            }
        }
        .frame(width: 24, height: 24)
    }
}

#Preview {
    @Previewable @State var location: AutomaticLocationNode = AutomaticLocationNode()

    AutomaticRelayItemView(
        location: location,
        onSelectLocation: {
            print("Selected: \($0.name)")
        },
        onSelectInfo: {
            print("Selected info")
        }
    )
}
