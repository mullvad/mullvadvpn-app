import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
protocol SelectLocationViewModel: ObservableObject {
    var activeLocationContext: LocationContext { get set }
    var multihopContext: MultihopContext? { get set }
    var searchText: String { get set }
    var showDAITAInfo: Bool { get }
    func onFilterTapped(_ filter: SelectLocationFilter)
    func onFilterRemoved(_ filter: SelectLocationFilter)
    func refreshCustomLists()
    func addLocationToCustomList(location: LocationNode, customListName: String)
    func removeLocationFromCustomList(location: LocationNode, customListName: String)
    func deleteCustomList(name: String)
    func editCustomList(name: String)
    func didFinish()
    func showDaitaSettings()
    func showEditCustomListView(locations: [LocationNode])
    func showAddCustomListView(locations: [LocationNode])
    func showFilterView()
}

struct SelectLocationDelegate {
    let showDaitaSettings: () -> Void
    let showObfuscationSettings: () -> Void
    let showFilterView: () -> Void
    let showEditCustomListView: ([LocationNode], CustomList?) -> Void
    let showAddCustomListView: ([LocationNode]) -> Void
    let didSelectExitRelayLocations: (UserSelectedRelays) -> Void
    let didSelectEntryRelayLocations: (UserSelectedRelays) -> Void
    let didFinish: () -> Void
}

@MainActor
class SelectLocationViewModelImpl: SelectLocationViewModel {
    var activeLocationContext: LocationContext {
        get {
            switch multihopContext {
            case .entry:
                entryContext
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
    @Published private var entryContext: LocationContext
    @Published var searchText: String = ""
    @Published var showDAITAInfo: Bool

    private let exitLocationsDataSource = AllLocationDataSource()
    private let entryLocationsDataSource = AllLocationDataSource()
    private let entryCustomListsDataSource: CustomListsDataSource
    private let exitCustomListsDataSource: CustomListsDataSource

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListInteractor: CustomListInteractorProtocol
    private var relaysCandidates: RelayCandidates?

    private var tunnelObserver: TunnelBlockObserver?

    let delegate: SelectLocationDelegate

    var cancellables: [Combine.AnyCancellable] = []

    init(
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        customListRepository: CustomListRepositoryProtocol,
        delegate: SelectLocationDelegate
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.customListInteractor = CustomListInteractor(
            tunnelManager: tunnelManager,
            repository: customListRepository
        )
        self.delegate = delegate
        self.entryCustomListsDataSource = CustomListsDataSource(
            repository: customListRepository
        )
        self.exitCustomListsDataSource = CustomListsDataSource(
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
            self.multihopContext =
                if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                    .blockedState?.reason
                { .entry } else { .exit }
        }

        showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

        self.exitContext = LocationContext(
            locations: [],
            customLists: [],
            filter:
                SelectLocationViewModelImpl
                .getActiveExitFilters(tunnelManager.settings),
            selectedLocation: nil,
            connectedRelayHostname: nil,
            selectLocation: { location in
                delegate.didSelectExitRelayLocations(getUserSelectedRelays(location))
            }
        )
        self.entryContext = LocationContext(
            locations: [],
            customLists: [],
            filter:
                SelectLocationViewModelImpl
                .getActiveEntryFilters(tunnelManager.settings),
            selectedLocation: nil,
            connectedRelayHostname: nil,
            selectLocation: { location in
                delegate.didSelectEntryRelayLocations(getUserSelectedRelays(location))
            }
        )
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    self?.updateConnectedLocation(status)
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    fetchLocations()
                    if settings.tunnelMultihopState.isEnabled {
                        if multihopContext == nil {
                            multihopContext = .exit
                        }
                        showDAITAInfo = settings.daita.isAutomaticRouting
                    } else {
                        multihopContext = nil
                    }
                    exitContext.filter =
                        SelectLocationViewModelImpl
                        .getActiveExitFilters(settings)
                    entryContext.filter =
                        SelectLocationViewModelImpl
                        .getActiveEntryFilters(settings)
                }
            )

        $searchText.receive(on: RunLoop.main).sink { [weak self] _ in
            self?.populateLocationLists()
        }.store(in: &cancellables)

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        updateConnectedLocation(tunnelManager.tunnelStatus)
        fetchLocations()
        populateLocationLists()
    }

    deinit {
        guard let tunnelObserver else { return }
        tunnelManager.removeObserver(tunnelObserver)
    }

    func onFilterTapped(_ filter: SelectLocationFilter) {
        switch filter {
        case .owned, .rented, .provider:
            delegate.showFilterView()
        case .daita:
            delegate.showDaitaSettings()
        case .obfuscation:
            delegate.showObfuscationSettings()
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

    func deleteCustomList(name: String) {
        guard let customList = customListInteractor.fetchAll().first(where: { $0.name == name }) else {
            return
        }
        customListInteractor.delete(customList: customList)
        populateLocationLists()
    }

    func editCustomList(name: String) {
        guard let customList = customListInteractor.fetchAll().first(where: { $0.name == name }) else {
            return
        }
        delegate
            .showEditCustomListView(activeLocationContext.locations, customList)
    }

    func refreshCustomLists() {
        populateLocationLists()
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
                    .append(.obfuscation)
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
            self.entryContext.connectedRelayHostname = hostname
        } else {
            self.entryContext.connectedRelayHostname = nil
        }
    }

    private func fetchLocations() {
        relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        )
    }

    private func populateLocationLists() {
        if let relaysCandidates {
            exitLocationsDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())
            exitContext.locations = exitLocationsDataSource.search(by: searchText)
            if let entryRelays = relaysCandidates.entryRelays {
                entryLocationsDataSource
                    .reload(entryRelays.toLocationRelays())
                entryContext.locations =
                    entryLocationsDataSource
                    .search(by: searchText)
            }
        }
        exitCustomListsDataSource.reload(allLocationNodes: exitContext.locations)
        exitContext.customLists =
            exitCustomListsDataSource
            .search(by: searchText)
            .map {
                newCustomList in
                let oldCustomList =
                    exitContext.customLists
                    .first { existingCustomList in
                        existingCustomList.code == newCustomList.code
                    }
                newCustomList.showsChildren = oldCustomList?.showsChildren ?? false
                newCustomList.children = newCustomList.children.map { newLocation in
                    let existingLocation = exitContext.customLists
                        .first {
                            oldLocation in
                            oldLocation.code == newLocation.code
                        }
                    newLocation.showsChildren = existingLocation?.showsChildren ?? false
                    return newLocation
                }
                return newCustomList
            }

        entryCustomListsDataSource.reload(allLocationNodes: entryContext.locations)
        entryContext.customLists =
            entryCustomListsDataSource
            .search(by: searchText)
            .map {
                newCustomList in
                let oldCustomList =
                    entryContext.customLists
                    .first { existingCustomList in
                        existingCustomList.code == newCustomList.code
                    }
                newCustomList.showsChildren = oldCustomList?.showsChildren ?? false
                newCustomList.children = newCustomList.children.map { newLocation in
                    let existingLocation = exitContext.customLists
                        .first {
                            oldLocation in
                            oldLocation.code == newLocation.code
                        }
                    newLocation.showsChildren = existingLocation?.showsChildren ?? false
                    return newLocation
                }
                return newCustomList
            }

        if let exitLocations = tunnelManager.settings.relayConstraints.exitLocations.value {
            setSelection(
                selectedExitRelays: exitLocations,
                selectedEntryRelays: tunnelManager.settings.relayConstraints.entryLocations.value
            )
        }
    }

    private func setSelection(
        selectedExitRelays: UserSelectedRelays,
        selectedEntryRelays: UserSelectedRelays?
    ) {
        if let customListSelection = selectedExitRelays.customListSelection,
            let customList = exitCustomListsDataSource.customList(by: customListSelection.listId)
        {
            exitContext.selectedLocation =
                exitCustomListsDataSource
                .node(by: selectedExitRelays, for: customList)
        } else if let location = selectedExitRelays.locations.first {
            exitContext.selectedLocation = exitLocationsDataSource.node(by: location)
        } else {
            exitContext.selectedLocation = nil
        }
        if let selectedEntryRelays {
            if let customListSelection = selectedEntryRelays.customListSelection,
                let customList = entryCustomListsDataSource.customList(by: customListSelection.listId)
            {
                entryContext.selectedLocation =
                    entryCustomListsDataSource
                    .node(by: selectedEntryRelays, for: customList)
            } else if let location = selectedEntryRelays.locations.first {
                entryContext.selectedLocation =
                    entryLocationsDataSource
                    .node(by: location)
            } else {
                entryContext.selectedLocation = nil
            }
        } else {
            entryContext.selectedLocation = nil
        }
        expandSelectedLocation()
    }

    private func expandSelectedLocation() {
        if var selectedExitLocation = exitContext.selectedLocation {
            while let parent = selectedExitLocation.parent {
                parent.showsChildren = true
                selectedExitLocation = parent
            }
        }
        if var selectedEntryLocation = entryContext.selectedLocation {
            while let parent = selectedEntryLocation.parent {
                parent.showsChildren = true
                selectedEntryLocation = parent
            }
        }
    }

    func addLocationToCustomList(location: LocationNode, customListName: String) {
        let customList =
            customListInteractor.fetchAll().first { $0.name == customListName }
            ?? CustomList(
                name: customListName,
                locations: []
            )

        let allLocations = (customList.locations + location.locations)
        let locations: [RelayLocation] =
            allLocations
            .filter { $0.ancestors.allSatisfy { !allLocations.contains($0) } }
            .reduce(
                [],
                { partialResult, location in
                    if !partialResult.contains(location) {
                        return partialResult + [location]
                    } else {
                        return partialResult
                    }
                })
        let newCustomList = CustomList(
            id: customList.id,
            name: customList.name,
            locations: locations
        )
        try? customListInteractor.save(list: newCustomList)
        populateLocationLists()
    }

    func removeLocationFromCustomList(
        location: LocationNode,
        customListName: String
    ) {
        let customList = customListInteractor.fetchAll().first { $0.name == customListName }
        guard let customList else {
            return
        }
        let allLocations = customList.locations.filter { !location.locations.contains($0) }
        let newCustomList = CustomList(
            id: customList.id,
            name: customList.name,
            locations: allLocations
        )
        try? customListInteractor.save(list: newCustomList)
        populateLocationLists()
    }

    func didFinish() {
        delegate.didFinish()
    }

    func showDaitaSettings() {
        delegate.showFilterView()
    }

    func showEditCustomListView(locations: [LocationNode]) {
        delegate.showEditCustomListView(locations, nil)
    }

    func showAddCustomListView(locations: [LocationNode]) {
        delegate.showAddCustomListView(locations)
    }

    func showFilterView() {
        delegate.showFilterView()
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
