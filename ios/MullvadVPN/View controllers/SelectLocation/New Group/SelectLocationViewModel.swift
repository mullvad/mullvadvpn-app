import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
class SelectLocationViewModelImpl: SelectLocationViewModel {
    @Published var connectedRelayHostname: String?

    @Published var customLists: [LocationNode] = []

    @Published var allLocations: [LocationNode] = []
    @Published var selectedLocation: LocationNode?

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListRepository: CustomListRepositoryProtocol

    private let didSelectRelayLocations: (UserSelectedRelays) -> Void
    let showFilterView: (() -> Void)?
    let showEditCustomListView: (([LocationNode]) -> Void)?
    let showAddCustomListView: (([LocationNode]) -> Void)?
    let didFinish: (() -> Void)?

    private var tunnelObserver: TunnelBlockObserver?

    init(
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        customListRepository: CustomListRepositoryProtocol,
        didSelectRelayLocations: @escaping (UserSelectedRelays) -> Void,
        showFilterView: @escaping (() -> Void),
        showEditCustomListView: @escaping (([LocationNode]) -> Void),
        showAddCustomListView: @escaping (([LocationNode]) -> Void),
        didFinish: @escaping (() -> Void)
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.customListRepository = customListRepository
        self.didSelectRelayLocations = didSelectRelayLocations
        self.showFilterView = showFilterView
        self.showEditCustomListView = showEditCustomListView
        self.showAddCustomListView = showAddCustomListView
        self.didFinish = didFinish

        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    self?.updateConnectedLocation(status)
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    fetchLocations(settings: settings)
                }
            )

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        updateConnectedLocation(tunnelManager.tunnelStatus)
        fetchLocations(settings: tunnelManager.settings)
    }

    deinit {
        guard let tunnelObserver else { return }
        tunnelManager.removeObserver(tunnelObserver)
    }

    fileprivate func updateConnectedLocation(_ status: TunnelStatus) {
        Task { @MainActor in
            if let hostname = status.state.relays?.exit.hostname {
                self.connectedRelayHostname = hostname
            } else {
                self.connectedRelayHostname = nil
            }
        }
    }

    private func fetchLocations(settings: LatestTunnelSettings) {
        let relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        )
        let allLocationDataSource = AllLocationDataSource()
        if let relaysCandidates {
            allLocationDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())
            allLocations = allLocationDataSource.nodes
        }
        let customListsDataSource = CustomListsDataSource(repository: customListRepository)
        customListsDataSource.reload(allLocationNodes: allLocations)
        customLists = customListsDataSource.nodes

        if let exitLocations = settings.relayConstraints.exitLocations.value {
            setSelection(
                selectedRelays: exitLocations,
                allLocationsDataSource: allLocationDataSource,
                customListsDataSource: customListsDataSource
            )
        }
    }

    private func setSelection(
        selectedRelays: UserSelectedRelays,
        allLocationsDataSource: AllLocationDataSource,
        customListsDataSource: CustomListsDataSource
    ) {
        if let customListSelection = selectedRelays.customListSelection,
           let customList = customListsDataSource.customList(by: customListSelection.listId),
           let selectedLocation = customListsDataSource
           .node(by: selectedRelays, for: customList) {
            self.selectedLocation = selectedLocation
        } else if let location = selectedRelays.locations.first,
                  let selectedLocation = allLocationsDataSource.node(by: location) {
            self.selectedLocation = selectedLocation
        }
    }

    func onSelectLocation(_ location: LocationNode) {
        var customListSelection: UserSelectedRelays.CustomListSelection?
        if let topmostNode = location.root as? CustomListLocationNode {
            customListSelection = UserSelectedRelays.CustomListSelection(
                listId: topmostNode.customList.id,
                isList: topmostNode == location
            )
        }

        let relayLocations = UserSelectedRelays(
            locations: location.locations,
            customListSelection: customListSelection
        )
        didSelectRelayLocations(relayLocations)
    }
}

class MockSelectLocationViewModel: SelectLocationViewModel {
    var connectedRelayHostname: String?

    var selectedLocation: LocationNode?

    var showFilterView: (() -> Void)?

    var showEditCustomListView: (([LocationNode]) -> Void)?

    var showAddCustomListView: (([LocationNode]) -> Void)?

    var didFinish: (() -> Void)?

    func onSelectLocation(_ location: LocationNode) {
        print("Selected location: \(location.name)")
    }

    var customLists: [LocationNode] = [
        LocationNode(name: "MyList1", code: "sth", children: [
            LocationNode(name: "Sweden", code: "se", children: [
                LocationNode(
                    name: "Stockholm",
                    code: "sth",
                    children: [
                        LocationNode(name: "se-sto-001", code: "se-sto-001"),
                        LocationNode(name: "se-sto-002", code: "se-sto-002"),
                        LocationNode(name: "se-sto-003", code: "se-sto-003"),
                    ]
                ),
                LocationNode(name: "Gothenburg", code: "gto", children: [
                    LocationNode(name: "se-got-001", code: "se-got-001"),
                    LocationNode(name: "se-got-002", code: "se-got-002"),
                    LocationNode(name: "se-got-003", code: "se-got-003"),
                ]),
            ]),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
            ]),
            LocationNode(name: "se-got-003", code: "se-got-003"),
        ]),
        LocationNode(name: "MyList2", code: "sth", children: [
            LocationNode(name: "Germany", code: "de", children: [
                LocationNode(name: "Berlin", code: "ber", children: [
                    LocationNode(name: "de-ber-001", code: "de-ber-001"),
                    LocationNode(name: "de-ber-002", code: "de-ber-002"),
                    LocationNode(name: "de-ber-003", code: "de-ber-003"),
                ]),
                LocationNode(name: "Frankfurt", code: "fra", children: [
                    LocationNode(name: "de-fra-001", code: "de-fra-001"),
                    LocationNode(name: "de-fra-002", code: "de-fra-002"),
                    LocationNode(name: "de-fra-003", code: "de-fra-003"),
                ]),
            ]),
        ]),
    ]

    var allLocations: [LocationNode] = [
        LocationNode(name: "Sweden", code: "se", children: [
            LocationNode(
                name: "Stockholm",
                code: "sth",
                children: [
                    LocationNode(name: "se-sto-001", code: "se-sto-001"),
                    LocationNode(name: "se-sto-002", code: "se-sto-002"),
                    LocationNode(name: "se-sto-003", code: "se-sto-003"),
                ]
            ),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
                LocationNode(name: "se-got-003", code: "se-got-003"),
            ]),
        ]),
        LocationNode(name: "Germany", code: "de", children: [
            LocationNode(name: "Berlin", code: "ber", children: [
                LocationNode(name: "de-ber-001", code: "de-ber-001"),
                LocationNode(name: "de-ber-002", code: "de-ber-002"),
                LocationNode(name: "de-ber-003", code: "de-ber-003"),
            ]),
            LocationNode(name: "Frankfurt", code: "fra", children: [
                LocationNode(name: "de-fra-001", code: "de-fra-001"),
                LocationNode(name: "de-fra-002", code: "de-fra-002"),
                LocationNode(name: "de-fra-003", code: "de-fra-003"),
            ]),
        ]),
        LocationNode(name: "France", code: "fr", children: [
            LocationNode(name: "Paris", code: "par", children: [
                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                LocationNode(name: "fr-par-003", code: "fr-par-003"),
            ]),
            LocationNode(name: "Lyon", code: "lyo", children: [
                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
            ]),
        ]),
    ]
}
