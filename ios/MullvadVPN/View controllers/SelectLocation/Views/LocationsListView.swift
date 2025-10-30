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
        VStack(spacing: 4) {
            ForEach(
                Array(filteredLocationIndices.enumerated()),
                id: \.element
            ) { index, indexInLocationList in
                let location = $locations[indexInLocationList]
                LocationListItem(
                    location: location,
                    multihopContext: multihopContext,
                    position: ItemPosition(
                        index: index,
                        count: filteredLocationIndices.count
                    ),
                    onSelect: onSelectLocation,
                    contextMenu: { location in contextMenu(location) }
                )
            }
        }
    }
}

@available(iOS 17, *)
#Preview {
    @Previewable @State var locations: [LocationNode] = [
        LocationNode(
            name: "Sweden", code: "se",
            children: [
                LocationNode(
                    name: "Stockholm",
                    code: "sth",
                    isActive: false,
                    children: [
                        LocationNode(name: "se-sto-001", code: "se-sto-001"),
                        LocationNode(name: "se-sto-002", code: "se-sto-002"),
                        LocationNode(name: "se-sto-003", code: "se-sto-003"),
                    ]
                ),
                LocationNode(
                    name: "Gothenburg", code: "gto",
                    children: [
                        LocationNode(name: "se-got-001", code: "se-got-001"),
                        LocationNode(name: "se-got-002", code: "se-got-002"),
                        LocationNode(name: "se-got-003", code: "se-got-003"),
                    ]),
            ]),
        LocationNode(name: "blo-la-003", code: "blo-la-003"),
        LocationNode(name: "blo-la-005", code: "blo-la-005", isActive: false),
        LocationNode(
            name: "Germany", code: "de",
            children: [
                LocationNode(
                    name: "Berlin", code: "ber",
                    children: [
                        LocationNode(name: "de-ber-001", code: "de-ber-001"),
                        LocationNode(name: "de-ber-002", code: "de-ber-002"),
                        LocationNode(name: "de-ber-003", code: "de-ber-003"),
                    ]),
                LocationNode(
                    name: "Frankfurt", code: "fra",
                    children: [
                        LocationNode(name: "de-fra-001", code: "de-fra-001"),
                        LocationNode(name: "de-fra-002", code: "de-fra-002"),
                        LocationNode(name: "de-fra-003", code: "de-fra-003"),
                    ]),
            ]),
        LocationNode(
            name: "France", code: "fr",
            children: [
                LocationNode(
                    name: "Paris", code: "par",
                    children: [
                        LocationNode(name: "fr-par-001", code: "fr-par-001"),
                        LocationNode(name: "fr-par-002", code: "fr-par-002"),
                        LocationNode(name: "fr-par-003", code: "fr-par-003"),
                    ]),
                LocationNode(
                    name: "Lyon", code: "lyo", isActive: false,
                    children: [
                        LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                        LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                        LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
                    ]),
            ]),
        LocationNode(name: "Lalala", code: "test"),
        LocationNode(
            name: "Custom list", code: "blda",
            children: [
                LocationNode(name: "de-ber-003", code: "de-ber-003"),

                LocationNode(
                    name: "France", code: "fr",
                    children: [
                        LocationNode(
                            name: "Paris", code: "par",
                            children: [
                                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                                LocationNode(name: "fr-par-003", code: "fr-par-003"),
                            ]),
                        LocationNode(
                            name: "Lyon", code: "lyo",
                            children: [
                                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
                            ]),
                    ]),
                LocationNode(name: "testserver", code: "1234"),
            ]),
    ]
    ScrollView {
        LocationsListView(
            locations: $locations,
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

@available(iOS 17, *)
#Preview {
    @Previewable @State var location = LocationNode(
        name: "Custom list", code: "blda",
        children: [
            LocationNode(name: "de-ber-003", code: "de-ber-003"),

            LocationNode(
                name: "France", code: "fr",
                children: [
                    LocationNode(
                        name: "Paris", code: "par",
                        children: [
                            LocationNode(name: "fr-par-001", code: "fr-par-001"),
                            LocationNode(name: "fr-par-002", code: "fr-par-002"),
                            LocationNode(name: "fr-par-003", code: "fr-par-003"),
                        ], showsChildren: true),
                    LocationNode(
                        name: "Lyon", code: "lyo",
                        children: [
                            LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                            LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                            LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
                        ]),
                ], showsChildren: true),
            LocationNode(name: "testserver", code: "1234"),
        ], showsChildren: true)
    ScrollView {
        LocationListItem(
            location: $location,
            multihopContext: .exit,
            position: .only,
            onSelect: { _ in },
            contextMenu: { location in Text("Add \(location.name) to list") },
            level: 0
        )
    }
}
