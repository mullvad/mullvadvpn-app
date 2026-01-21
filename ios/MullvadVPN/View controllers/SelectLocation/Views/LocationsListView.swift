import SwiftUI

struct LocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let onExpand: ((LocationNode) -> Void)?
    let contextMenu: (LocationNode) -> ContextMenu

    init(
        locations: Binding<[LocationNode]>,
        multihopContext: MultihopContext,
        onSelectLocation: @escaping (LocationNode) -> Void,
        onExpand: ((LocationNode) -> Void)? = nil,
        contextMenu: @escaping (LocationNode) -> ContextMenu
    ) {
        self._locations = locations
        self.multihopContext = multihopContext
        self.onSelectLocation = onSelectLocation
        self.onExpand = onExpand
        self.contextMenu = contextMenu
    }

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .filter { !$0.element.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
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
                onExpand: onExpand,
                contextMenu: { location in contextMenu(location) }
            )
        }
    }
}

#Preview {
    @Previewable @StateObject var viewModel = MockSelectLocationViewModel()
    ScrollView {
        LazyVStack(spacing: 0) {
            LocationsListView(
                locations: $viewModel.exitContext.customLists,
                multihopContext: .exit,
                onSelectLocation: { location in
                    print("Selected: \(location.name)")
                },
                contextMenu: { location in Text("Add \(location.name) to list") }
            )
            .padding(.horizontal)
        }
    }
    .background(Color.mullvadBackground)
}
