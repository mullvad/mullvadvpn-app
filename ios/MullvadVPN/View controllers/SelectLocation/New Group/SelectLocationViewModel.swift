import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
class SelectLocationViewModelImpl: SelectLocationViewModel {
    @Published var activeFilter: [SelectLocationFilter]
    @Published var connectedRelayHostname: String?
    @Published var customLists: [LocationNode] = []
    @Published var allLocations: [LocationNode] = []
    @Published var selectedLocation: LocationNode?
    @Published var searchText: String = ""

    private let allLocationDataSource = AllLocationDataSource()
    private let customListsDataSource: CustomListsDataSource

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListRepository: CustomListRepositoryProtocol

    private let didSelectRelayLocations: (UserSelectedRelays) -> Void
    let showFilterView: (() -> Void)?
    let showEditCustomListView: (([LocationNode]) -> Void)?
    let showAddCustomListView: (([LocationNode]) -> Void)?
    let didFinish: (() -> Void)?

    private var tunnelObserver: TunnelBlockObserver?

    var cancellables: [Combine.AnyCancellable] = []
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
        self.customListsDataSource = CustomListsDataSource(
            repository: customListRepository
        )
        activeFilter = SelectLocationViewModelImpl
            .getActiveFilters(tunnelManager.settings)
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

                    activeFilter = SelectLocationViewModelImpl.getActiveFilters(settings)
                }
            )

        $searchText.receive(on: RunLoop.main).sink { [weak self] searchText in
            self?.filterBySearch(searchText)
        }.store(in: &cancellables)

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        updateConnectedLocation(tunnelManager.tunnelStatus)
        fetchLocations(settings: tunnelManager.settings)
    }

    deinit {
        guard let tunnelObserver else { return }
        tunnelManager.removeObserver(tunnelObserver)
    }

    func onFilterTapped(_ filter: SelectLocationFilter) {
        switch filter {
        case .owned, .rented, .provider:
            showFilterView?()
        default:
            break
        }
    }

    func onFilterRemoved(_ filter: SelectLocationFilter) {
        switch filter {
        case .owned, .rented:
            var relayConstraints = tunnelManager.settings.relayConstraints
            guard var filter = relayConstraints.filter.value else { return }
            filter.ownership = .any
            relayConstraints.filter = .only(filter)
            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
        case .provider:
            var relayConstraints = tunnelManager.settings.relayConstraints
            guard var filter = relayConstraints.filter.value else { return }
            filter.providers = .any
            relayConstraints.filter = .only(filter)
            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
        default:
            break
        }
    }

    func refreshCustomLists() {
        fetchLocations(settings: tunnelManager.settings)
    }

    private static func getActiveFilters(_ settings: LatestTunnelSettings) -> [SelectLocationFilter] {
        var activeFilter: [SelectLocationFilter] = []

        if let ownershipFilter = settings.relayConstraints.filter.value {
            switch ownershipFilter.ownership {
            case .any:
                break
            case .owned:
                activeFilter.append(.owned)
            case .rented:
                activeFilter.append(.rented)
            }
            if let provider = ownershipFilter.providers.value {
                activeFilter.append(.provider(provider.count))
            }
        }
        if settings.daita.isDirectOnly {
            activeFilter.append(.daita)
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            activeFilter
                .append(
                    .obfuscation
                )
        }
        return activeFilter
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
        if let relaysCandidates {
            allLocationDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())
            allLocations = allLocationDataSource.search(by: searchText)
        }
        customListsDataSource.reload(allLocationNodes: allLocations)
        customLists = customListsDataSource.search(by: searchText)

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
        var selectedLocation: LocationNode?
        if let customListSelection = selectedRelays.customListSelection,
           let customList = customListsDataSource.customList(by: customListSelection.listId) {
            selectedLocation = customListsDataSource
                .node(by: selectedRelays, for: customList)
        } else if let location = selectedRelays.locations.first {
            selectedLocation = allLocationsDataSource.node(by: location)
        }
        self.selectedLocation = selectedLocation
        expandSelectedLocation()
    }

    func expandSelectedLocation() {
        if var selectedLocation {
            while let parent = selectedLocation.parent {
                parent.showsChildren = true
                selectedLocation = parent
            }
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

    func filterBySearch(_ query: String) {
        allLocations = allLocationDataSource.search(by: query)
        customLists = customListsDataSource.search(by: query)
        expandSelectedLocation()
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
