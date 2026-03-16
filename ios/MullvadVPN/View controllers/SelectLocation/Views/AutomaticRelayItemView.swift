import SwiftUI

struct AutomaticRelayItemView: View {
    let location: LocationNode
    var subtitle: LocalizedStringKey?
    let onSelectLocation: (LocationNode) -> Void
    let onSelectInfo: () -> Void

    var body: some View {
        SegmentedListItem(
            title: LocalizedStringKey(location.name),
            subtitle: subtitle,
            secondaryButtonImage: .mullvadIconInfo,
            onSelect: { onSelectLocation(location) },
            onSecondarySelect: onSelectInfo,
            minHeight: UIMetrics.LocationList.cellMinHeight
        )
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
