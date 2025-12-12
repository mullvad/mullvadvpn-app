import Foundation

class MockSelectLocationViewModel: SelectLocationViewModel {
    var isMultihopEnabled: Bool = true

    var showDAITAInfo = false

    var entryContext: LocationContext
    var exitContext: LocationContext

    @Published var multihopContext: MultihopContext = .exit

    init() {
        entryContext = LocationContext()
        exitContext = LocationContext(
            locations: [
                LocationNode(
                    name: "Sweden", code: "se",
                    children: [
                        LocationNode(
                            name: "Stockholm",
                            code: "sth",
                            children: [
                                LocationNode(name: sth1, code: sth1),
                                LocationNode(name: sth2, code: sth2),
                                LocationNode(name: sth3, code: sth3),
                            ], showsChildren: true
                        ),
                        LocationNode(
                            name: "Gothenburg", code: "got",
                            children: [
                                LocationNode(name: got1, code: got1),
                                LocationNode(name: got2, code: got2),
                                LocationNode(name: got3, code: got3),
                            ], showsChildren: true),
                    ], showsChildren: true),
                LocationNode(
                    name: "Germany", code: "de",
                    children: [
                        LocationNode(
                            name: "Berlin", code: "ber",
                            children: [
                                LocationNode(name: ber1, code: ber1),
                                LocationNode(name: ber2, code: ber2),
                                LocationNode(name: ber3, code: ber3),
                            ], showsChildren: true),
                        LocationNode(
                            name: "Frankfurt", code: "fra",
                            children: [
                                LocationNode(name: fra1, code: fra1),
                                LocationNode(name: fra2, code: fra2),
                                LocationNode(name: fra3, code: fra3),
                            ], showsChildren: true),
                    ], showsChildren: true),
                LocationNode(
                    name: "France", code: "fr",
                    children: [
                        LocationNode(
                            name: "Paris", code: "par",
                            children: [
                                LocationNode(name: par1, code: par1),
                                LocationNode(name: par2, code: par2),
                                LocationNode(name: par3, code: par3),
                            ], showsChildren: true),
                        LocationNode(
                            name: "Lyon", code: "lyo",
                            children: [
                                LocationNode(name: lyo1, code: lyo1),
                                LocationNode(name: lyo2, code: lyo2),
                                LocationNode(name: lyo3, code: lyo3),
                            ], showsChildren: true),
                    ], showsChildren: true),
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
                                    name: "Gothenburg", code: "got",
                                    children: [
                                        LocationNode(name: got1, code: got1),
                                        LocationNode(name: got2, code: got2),
                                        LocationNode(name: got3, code: got3),
                                    ]),
                            ]),
                        LocationNode(
                            name: "Gothenburg", code: "got",
                            children: [
                                LocationNode(name: got1, code: got1),
                                LocationNode(name: got2, code: got2),
                            ]),
                        LocationNode(name: got3, code: got3),
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
                                        LocationNode(name: ber1, code: ber1),
                                        LocationNode(name: ber2, code: ber2),
                                        LocationNode(name: ber3, code: ber3),
                                    ]),
                                LocationNode(
                                    name: "Frankfurt", code: "fra",
                                    children: [
                                        LocationNode(name: fra1, code: fra1),
                                        LocationNode(name: fra2, code: fra2),
                                        LocationNode(name: fra3, code: fra3),
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
    }

    func toggleMultihop() { isMultihopEnabled.toggle() }

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

    private var got1 = "se-got-001"
    private let got2 = "se-got-002"
    private let got3 = "se-got-003"
    private let sth1 = "se-sto-001"
    private let sth2 = "se-sto-002"
    private let sth3 = "se-sto-003"
    private let ber1 = "de-ber-001"
    private let ber2 = "de-ber-002"
    private let ber3 = "de-ber-003"
    private let fra1 = "de-fra-001"
    private let fra2 = "de-fra-002"
    private let fra3 = "de-fra-003"
    private let par1 = "fr-par-001"
    private let par2 = "fr-par-002"
    private let par3 = "fr-par-003"
    private let lyo1 = "fr-lyo-001"
    private let lyo2 = "fr-lyo-002"
    private let lyo3 = "fr-lyo-003"
}
