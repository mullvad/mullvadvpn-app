import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
class SelectLocationViewModelImpl: SelectLocationViewModel {
    var activeLocationContext: LocationContext {
        get {
            switch multihopContext {
            case .entry:
                entryContext!
            case .exit, nil:
                exitContext
            }
        }
        set {
            switch multihopContext {
            case .entry:
                entryContext = newValue
            case .exit, nil:
                exitContext = newValue
            }
        }
    }

    @Published var multihopContext: MultihopContext?

//    @Published var activeEntryFilter: [SelectLocationFilter]
//    @Published var connectedExitRelayHostname: String?
//    @Published var connectedEntryRelayHostname: String?
//    @Published var customLists: [LocationNode] = []
    @Published private var exitContext: LocationContext
    @Published private var entryContext: LocationContext?
//    @Published var selectedExitLocation: LocationNode?
//    @Published var selectedEntryLocation: LocationNode?
    @Published var searchText: String = ""
    @Published var context: LocationContext?

    private let exitLocationsDataSource = AllLocationDataSource()
    private let entryLocationsDataSource = AllLocationDataSource()
    private let customListsDataSource: CustomListsDataSource

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListRepository: CustomListRepositoryProtocol

    private let didSelectExitRelayLocations: (UserSelectedRelays) -> Void
    private let didSelectEntryRelayLocations: (UserSelectedRelays) -> Void
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
        didSelectExitRelayLocations: @escaping (UserSelectedRelays) -> Void,
        didSelectEntryRelayLocations: @escaping (UserSelectedRelays) -> Void,
        showFilterView: @escaping (() -> Void),
        showEditCustomListView: @escaping (([LocationNode]) -> Void),
        showAddCustomListView: @escaping (([LocationNode]) -> Void),
        didFinish: @escaping (() -> Void)
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.customListRepository = customListRepository
        self.didSelectExitRelayLocations = didSelectExitRelayLocations
        self.didSelectEntryRelayLocations = didSelectEntryRelayLocations
        self.showFilterView = showFilterView
        self.showEditCustomListView = showEditCustomListView
        self.showAddCustomListView = showAddCustomListView
        self.didFinish = didFinish
        self.customListsDataSource = CustomListsDataSource(
            repository: customListRepository
        )
        func getUserSelectedRelays(_ location: LocationNode) -> UserSelectedRelays {
            var customListSelection: UserSelectedRelays.CustomListSelection?
            if let topmostNode = location.root as? CustomListLocationNode {
                customListSelection = UserSelectedRelays.CustomListSelection(
                    listId: topmostNode.customList.id,
                    isList: topmostNode == location
                )
            }

            return UserSelectedRelays(
                locations: location.locations,
                customListSelection: customListSelection
            )
        }

        // If multihop is enabled, we should check if there's a DAITA related error when opening the location
        // view. If there is, help the user by showing the entry instead of the exit view.
        if tunnelManager.settings.tunnelMultihopState.isEnabled {
            self.multihopContext = if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                .blockedState?.reason { .entry } else { .exit }
        }
        self.exitContext = LocationContext(
            locations: [],
            customLists: [],
            filter: SelectLocationViewModelImpl
                .getActiveEntryFilters(tunnelManager.settings),
            selectedLocation: nil,
            connectedRelayHostname: nil,
            selectLocation: { location in
                didSelectExitRelayLocations(getUserSelectedRelays(location))
            }
        )
        self.entryContext = LocationContext(
            locations: [],
            customLists: [],
            filter: SelectLocationViewModelImpl
                .getActiveEntryFilters(tunnelManager.settings),
            selectedLocation: nil,
            connectedRelayHostname: nil,
            selectLocation: { location in
                didSelectEntryRelayLocations(getUserSelectedRelays(location))
            }
        )
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    self?.updateConnectedLocation(status)
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    fetchLocations(settings: settings)
                    let isMultihop = settings.tunnelMultihopState.isEnabled
                    if isMultihop {
                        if multihopContext == nil {
                            multihopContext = .exit
                        }
                    } else {
                        multihopContext = nil
                    }
//                    let isAutomaticRouting = settings.daita.isAutomaticRouting

//                    activeEntryFilter = SelectLocationViewModelImpl.getActiveFilters(settings)
                }
            )

        $searchText.receive(on: RunLoop.main).sink { [weak self] _ in
            self?.filterBySearch()
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

    private static func getActiveEntryFilters(_ settings: LatestTunnelSettings) -> [SelectLocationFilter] {
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
            if settings.tunnelMultihopState.isEnabled {
                activeFilter.append(.daita)
            }
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
        if let hostname = status.state.relays?.exit.hostname {
            self.exitContext.connectedRelayHostname = hostname
        } else {
            self.exitContext.connectedRelayHostname = nil
        }
        if let hostname = status.state.relays?.entry?.hostname {
            self.entryContext?.connectedRelayHostname = hostname
        } else {
            self.entryContext?.connectedRelayHostname = nil
        }
    }

    private func fetchLocations(settings: LatestTunnelSettings) {
        let relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        )
        if let relaysCandidates {
            exitLocationsDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())
            exitContext.locations = exitLocationsDataSource.search(by: searchText)
            if let entryRelays = relaysCandidates.entryRelays {
                entryLocationsDataSource
                    .reload(entryRelays.toLocationRelays())
                entryContext?.locations = entryLocationsDataSource
                    .search(by: searchText)
            }
        }
        customListsDataSource.reload(allLocationNodes: exitContext.locations)
        exitContext.customLists = customListsDataSource.search(by: searchText)
        if let entryContext {
            customListsDataSource.reload(allLocationNodes: entryContext.locations)
            entryContext.customLists = customListsDataSource.search(by: searchText)
        }

        if let exitLocations = settings.relayConstraints.exitLocations.value {
            setSelection(
                selectedExitRelays: exitLocations,
                selectedEntryRelays: settings.relayConstraints.entryLocations.value
            )
        }
    }

    private func setSelection(
        selectedExitRelays: UserSelectedRelays,
        selectedEntryRelays: UserSelectedRelays?
    ) {
        if let customListSelection = selectedExitRelays.customListSelection,
           let customList = customListsDataSource.customList(by: customListSelection.listId) {
            exitContext.selectedLocation = customListsDataSource
                .node(by: selectedExitRelays, for: customList)
        } else if let location = selectedExitRelays.locations.first {
            exitContext.selectedLocation = exitLocationsDataSource.node(by: location)
        } else {
            exitContext.selectedLocation = nil
        }
        if let selectedEntryRelays {
            if let customListSelection = selectedEntryRelays.customListSelection,
               let customList = customListsDataSource.customList(by: customListSelection.listId) {
                entryContext?.selectedLocation = customListsDataSource
                    .node(by: selectedEntryRelays, for: customList)
            } else if let location = selectedEntryRelays.locations.first {
                entryContext?.selectedLocation = entryLocationsDataSource
                    .node(by: location)
            } else {
                entryContext?.selectedLocation = nil
            }
        } else {
            entryContext?.selectedLocation = nil
        }
        expandSelectedLocation()
    }

    func expandSelectedLocation() {
        if var selectedExitLocation = exitContext.selectedLocation {
            while let parent = selectedExitLocation.parent {
                parent.showsChildren = true
                selectedExitLocation = parent
            }
        }
        if var selectedEntryLocation = entryContext?.selectedLocation {
            while let parent = selectedEntryLocation.parent {
                parent.showsChildren = true
                selectedEntryLocation = parent
            }
        }
    }

    func filterBySearch() {
        fetchLocations(settings: tunnelManager.settings)
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
