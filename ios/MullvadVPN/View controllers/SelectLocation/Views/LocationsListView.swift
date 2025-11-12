import SwiftUI

struct LocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .filter { !$0.element.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
        LazyVStack(spacing: 4) {
            ForEach(
                Array(filteredLocationIndices.enumerated()),
                id: \.element
            ) {
                index,
                indexInLocationList in
                let location = $locations[indexInLocationList]
                LocationListItem(
                    location: location,
                    multihopContext: multihopContext,
                    onSelect: onSelectLocation,
                    contextMenu: { location in contextMenu(location) }
                )
            }
        }
    }
}

@available(iOS 17, *)
#Preview {
    @Previewable @StateObject var viewModel = MockSelectLocationViewModel()
    ScrollView {
        LocationsListView(
            locations: $viewModel.exitContext.locations,
            multihopContext: .exit,
            onSelectLocation: { location in
                print("Selected: \(location.name)")
            },
            contextMenu: { location in Text("Add \(location.name) to list") }
        )
        .padding()
    }
    .background(Color.mullvadBackground)
}
