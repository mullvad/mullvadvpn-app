import MullvadTypes
import SwiftUI

struct LocationsListView<ContextMenu: View>: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    @ViewBuilder let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        ForEach($locations, id: \.self) { location in
            LocationListItem(
                location: location,
                multihopContext: multihopContext,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) }
            )
        }
    }
}

#Preview {
    @Previewable @State var viewModel = MockSelectLocationViewModel()
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
    .onAppear {
        viewModel.exitContext.recents.insert(AutomaticLocationNode(), at: 0)
    }
}
