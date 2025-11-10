import Foundation

class MockSelectLocationViewModel: SelectLocationViewModel {
    var isMultihopEnabled: Bool = true

    var showDAITAInfo = false

    var entryContext = LocationContext()
    var exitContext: LocationContext = .init(
        locations: [
            LocationNode(
                name: "Sweden", code: "se",
                children: [
                    LocationNode(
                        name: "Stockholm",
                        code: "sth",
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
                ], showsChildren: true),
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
                        name: "Lyon", code: "lyo",
                        children: [
                            LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                            LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                            LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
                        ]),
                ]),
        ],
        customLists: [
            LocationNode(
                name: "MyList1", code: "mylist1",
                children: [
                    LocationNode(
                        name: "Sweden", code: "se",
                        children: [
                            LocationNode(
                                name: "Stockholm",
                                code: "sth",
                            ),
                            LocationNode(
                                name: "Gothenburg", code: "gto",
                                children: [
                                    LocationNode(name: "se-got-001", code: "se-got-001"),
                                    LocationNode(name: "se-got-002", code: "se-got-002"),
                                    LocationNode(name: "se-got-003", code: "se-got-003"),
                                ]),
                        ]),
                    LocationNode(
                        name: "Gothenburg", code: "gto",
                        children: [
                            LocationNode(name: "se-got-001", code: "se-got-001"),
                            LocationNode(name: "se-got-002", code: "se-got-002"),
                        ]),
                    LocationNode(name: "se-got-003", code: "se-got-003"),
                ]),
            LocationNode(
                name: "MyList2", code: "mylist2",
                children: [
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
                        ])
                ]),
            LocationNode(
                name: "Stockholm",
                code: "sth",
            ),
        ],
        filter: [
            .daita,
            .obfuscation,
            .rented,
            .owned,
            .provider(12),
        ],
        selectedLocation: nil,
        connectedRelayHostname: nil
    ) { _ in
    }

    @Published var multihopContext: MultihopContext = .entry

    func onFilterTapped(_ filter: SelectLocationFilter) {
        print("show filter: \(filter)")
    }

    func onFilterRemoved(_ filter: SelectLocationFilter) {
        print("remove filter: \(filter)")
    }

    var searchText: String = ""

    func customListsChanged() {}

    func addLocationToCustomList(
        location: LocationNode,
        customListName: String
    ) {}

    func deleteCustomList(name: String) {}

    func showEditCustomList(name: String) {}

    func removeLocationFromCustomList(
        location: LocationNode,
        customListName: String
    ) {}

    func didFinish() {}

    func showDaitaSettings() {}

    func showEditCustomListView(locations: [LocationNode]) {}

    func showAddCustomListView(locations: [LocationNode]) {}

    func showFilterView() {}

    func expandSelectedLocation() {}
}
