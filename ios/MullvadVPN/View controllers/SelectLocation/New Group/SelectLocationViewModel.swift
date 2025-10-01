import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

struct LocationDataSources {
    let recentLocationsDataSource: RecentsConnectionDataSource
    let customListsDataSource: CustomListsDataSource
    let allLocationsDataSource: AllLocationDataSource
}

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
    @Published private var exitContext: LocationContext
    @Published private var entryContext: LocationContext?
    @Published var searchText = ""
    @Published var context: LocationContext?

    private let exitLocationsDataSource: LocationDataSources
    private let entryLocationsDataSource: LocationDataSources
    private let recentConnectionsRepository: RecentConnectionRepositoryProtocol

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager

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
        recentConnectionsRepository: RecentConnectionRepositoryProtocol,
        didSelectExitRelayLocations: @escaping (UserSelectedRelays) -> Void,
        didSelectEntryRelayLocations: @escaping (UserSelectedRelays) -> Void,
        showFilterView: @escaping (() -> Void),
        showEditCustomListView: @escaping (([LocationNode]) -> Void),
        showAddCustomListView: @escaping (([LocationNode]) -> Void),
        didFinish: @escaping (() -> Void)
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.recentConnectionsRepository = recentConnectionsRepository

        let createDataSource: (
            _ customListRepository: CustomListRepositoryProtocol,
            _ recentConnectionsRepository: RecentConnectionRepositoryProtocol
        ) -> LocationDataSources = { customListRepository, recentConnectionsRepository in
            let customListsDataSource = CustomListsDataSource(repository: customListRepository)
            let allLocationsDataSource = AllLocationDataSource()

            return LocationDataSources(
                recentLocationsDataSource: RecentsConnectionDataSource(
                    repository: recentConnectionsRepository,
                    settings: tunnelManager.settings,
                    userSelectedLocationFinder: UserSelectedLocationFinding(
                        allLocationsDataSource: allLocationsDataSource,
                        customListsDataSource: customListsDataSource
                    )
                ),
                customListsDataSource: customListsDataSource,
                allLocationsDataSource: allLocationsDataSource
            )
        }

        self.entryLocationsDataSource = createDataSource(
            customListRepository,
            recentConnectionsRepository
        )
        self.exitLocationsDataSource = createDataSource(
            customListRepository,
            recentConnectionsRepository
        )

        self.didSelectExitRelayLocations = didSelectExitRelayLocations
        self.didSelectEntryRelayLocations = didSelectEntryRelayLocations
        self.showFilterView = showFilterView
        self.showEditCustomListView = showEditCustomListView
        self.showAddCustomListView = showAddCustomListView
        self.didFinish = didFinish

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
            self.multihopContext = if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus
                .observedState
                .blockedState?.reason { .entry } else { .exit }
        }
        self.exitContext = LocationContext(
            locations: [],
            customLists: [],
            filter: SelectLocationViewModelImpl
                .getActiveExitFilters(tunnelManager.settings),
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

        addObservers()
        load()
    }

    private func addObservers() {
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
                    exitContext.filter = SelectLocationViewModelImpl
                        .getActiveExitFilters(settings)
                    entryContext?.filter = SelectLocationViewModelImpl
                        .getActiveEntryFilters(settings)
                }
            )

        $searchText.receive(on: RunLoop.main).sink { [weak self] _ in
            self?.filterBySearch()
        }.store(in: &cancellables)

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    private func load() {
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

        let isMultihop = settings.tunnelMultihopState.isEnabled
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
            if isMultihop {
                activeFilter.append(.daita)
            }
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            if isMultihop {
                activeFilter
                    .append(
                        .obfuscation
                    )
            }
        }
        return activeFilter
    }

    private static func getActiveExitFilters(_ settings: LatestTunnelSettings) -> [SelectLocationFilter] {
        var activeFilter: [SelectLocationFilter] = []

        let isMultihop = settings.tunnelMultihopState.isEnabled
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
            if !isMultihop {
                activeFilter.append(.daita)
            }
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            if !isMultihop {
                activeFilter
                    .append(
                        .obfuscation
                    )
            }
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
        defer {
            if let selectedExitRelays = settings.relayConstraints.exitLocations.value {
                setSelection(
                    selectedExitRelays: selectedExitRelays,
                    selectedEntryRelays: settings.relayConstraints.entryLocations.value
                )
            }
        }

        guard let relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        ) else {
            return
        }
        exitLocationsDataSource.allLocationsDataSource.reload(relaysCandidates.exitRelays.toLocationRelays())
        exitContext.locations = exitLocationsDataSource.allLocationsDataSource.search(by: searchText)
        exitLocationsDataSource.customListsDataSource.reload(allLocationNodes: exitContext.locations)
        exitContext.customLists = exitLocationsDataSource.customListsDataSource.search(by: searchText)
        exitLocationsDataSource.recentLocationsDataSource.reload(allLocationNodes: exitContext.locations)
        exitContext.recentConnections = exitLocationsDataSource.recentLocationsDataSource.search(by: searchText)

        guard let entryRelays = relaysCandidates.entryRelays, var entryContext = self.entryContext else {
            return
        }

        entryLocationsDataSource.allLocationsDataSource.reload(entryRelays.toLocationRelays())
        entryContext.locations = entryLocationsDataSource.allLocationsDataSource.search(by: searchText)
        entryLocationsDataSource.customListsDataSource.reload(allLocationNodes: entryContext.locations)
        entryContext.customLists = entryLocationsDataSource.customListsDataSource.search(by: searchText)
        entryLocationsDataSource.recentLocationsDataSource.reload(allLocationNodes: entryContext.locations)
        entryContext.recentConnections = entryLocationsDataSource.recentLocationsDataSource.search(by: searchText)
    }

    private func setSelection(
        selectedExitRelays: UserSelectedRelays,
        selectedEntryRelays: UserSelectedRelays?
    ) {
        exitContext.selectedLocation = UserSelectedLocationFinding(
            allLocationsDataSource: exitLocationsDataSource.allLocationsDataSource,
            customListsDataSource: exitLocationsDataSource.customListsDataSource
        ).node(selectedExitRelays)
        entryContext?.selectedLocation = UserSelectedLocationFinding(
            allLocationsDataSource: entryLocationsDataSource.allLocationsDataSource,
            customListsDataSource: entryLocationsDataSource.customListsDataSource
        ).node(selectedExitRelays)

//        didSelectExitRelayLocations(getUserSelectedRelays(location))
        expandSelectedLocation()
    }

    private func getUserSelectedRelays(_ location: LocationNode) -> UserSelectedRelays {
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

    func saveRecentConnections() {
        Task { [weak self] in
            guard let self else { return }
            let maxLimit = 50
            if tunnelManager.settings.tunnelMultihopState.isEnabled,
               let entrySelectedLocation = entryContext?.selectedLocation,
               let exitSelectedLocation = exitContext.selectedLocation {
                await recentConnectionsRepository.add(
                    RecentConnection(
                        entry: getUserSelectedRelays(entrySelectedLocation),
                        exit: getUserSelectedRelays(exitSelectedLocation)
                    ),
                    maxLimit: maxLimit
                )
            } else if let exitSelectedLocation = exitContext.selectedLocation {
                await recentConnectionsRepository.add(
                    RecentConnection(exit: getUserSelectedRelays(exitSelectedLocation)),
                    maxLimit: maxLimit
                )
            }
        }
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
