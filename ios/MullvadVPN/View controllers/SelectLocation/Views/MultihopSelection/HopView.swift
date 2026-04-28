import MullvadTypes
import SwiftUI

struct HopView: View {
    let hop: Hop
    let isSelected: Bool
    let onFilterTapped: (MultihopContext) -> Void
    let onIconPositionChange: (CGRect) -> Void

    @ScaledMetric(wrappedValue: 24, relativeTo: .caption2) var iconWidth: CGFloat

    var body: some View {
        let hasFilters = hop.filterCount > 0
        let automaticMultihopIsActive =
            (hop.multihopState == .always || hop.multihopState == .whenNeeded)
            && hop.selectedLocation is AutomaticLocationNode

        HStack {
            let name =
                if let location = hop.selectedLocation {
                    if let automaticLocationCountry = location.asAutomaticLocationNode?.locationInfo?.first {
                        String(
                            format: NSLocalizedString(
                                "%@ (%@)",
                                comment: "Selected location name, with country in parentheses"
                            ),
                            location.name,
                            automaticLocationCountry
                        )
                    } else {
                        location.name
                    }
                } else {
                    "Select location"
                }

            hop.icon
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 18)
                .accessibilityHidden(true)
                .capturePosition(
                    in: .multihopSelection
                ) { position in
                    onIconPositionChange(position)
                }
            Text(LocalizedStringKey(name))
                .lineLimit(nil)
                .fixedSize(horizontal: false, vertical: true)
            Spacer()
            Button {
                onFilterTapped(hop.multihopContext)
            } label: {
                let icon: Image =
                    if hasFilters {
                        automaticMultihopIsActive
                            ? .mullvadIconFilterCutoutDisabled
                            : .mullvadIconFilterCutout
                    } else {
                        automaticMultihopIsActive
                            ? .mullvadIconFilterDisabled
                            : .mullvadIconFilter
                    }
                icon
                    .resizable()
                    .frame(width: iconWidth, height: iconWidth)
            }
            .accessibilityIdentifier(
                hop.multihopContext == .entry ? .selectLocationEntryFilterButton : .selectLocationExitFilterButton
            )
        }
        .font(.mullvadSmallSemiBold)
        .foregroundStyle(
            isSelected
                ? Color.mullvadTextPrimary
                : Color.mullvadTextSecondary
        )
        .padding(8)
    }
}

#Preview {
    HopView(
        hop: Hop(
            multihopContext: .entry,
            multihopState: .whenNeeded,
            selectedLocation: .init(name: "Sweden", code: "se"),
            filterCount: 1
        ),
        isSelected: true,
        onFilterTapped: { _ in },
        onIconPositionChange: { _ in }
    )
    .background(Color.MullvadList.Item.child3)
}
