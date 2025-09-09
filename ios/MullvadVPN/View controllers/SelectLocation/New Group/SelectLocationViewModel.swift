import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
class SelectLocationViewModelImpl: SelectLocationViewModel {
    @Published var activeFilter: [SelectLocationFilter] = []
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
                    let isMultihop = settings.tunnelMultihopState.isEnabled
                    if isMultihop {}
                    let isAutomaticRouting = settings.daita.isAutomaticRouting

                    updateActiveFilters(settings)
                }
            )

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        updateConnectedLocation(tunnelManager.tunnelStatus)
        fetchLocations(settings: tunnelManager.settings)
        updateActiveFilters(tunnelManager.settings)
    }

    deinit {
        guard let tunnelObserver else { return }
        tunnelManager.removeObserver(tunnelObserver)
    }

    private func updateActiveFilters(_ settings: LatestTunnelSettings) {
        var activeFilter: [SelectLocationFilter] = []

        if settings.daita.isDirectOnly {
            activeFilter.append(.daita)
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            activeFilter.append(.obfuscation)
        }
        self.activeFilter = activeFilter
    }

    private func updateConnectedLocation(_ status: TunnelStatus) {
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

private extension WireGuardObfuscationState {
    /// This flag affects whether the "Setting: Obfuscation" pill is shown when selecting a location
    var affectsRelaySelection: Bool {
        switch self {
        case .shadowsocks, .quic:
            true
        default: false
        }
    }
}
