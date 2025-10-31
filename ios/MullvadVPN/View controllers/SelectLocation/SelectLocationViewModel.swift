import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes

@MainActor
protocol SelectLocationViewModel: ObservableObject {
    var exitContext: LocationContext { get set }
    var entryContext: LocationContext { get set }
    var multihopContext: MultihopContext { get set }
    var searchText: String { get set }
    var showDAITAInfo: Bool { get }
    var isMultihopEnabled: Bool { get }
    func onFilterTapped(_ filter: SelectLocationFilter)
    func onFilterRemoved(_ filter: SelectLocationFilter)
    func refreshCustomLists()
    func addLocationToCustomList(location: LocationNode, customListName: String)
    func removeLocationFromCustomList(location: LocationNode, customListName: String)
    func deleteCustomList(name: String)
    func showEditCustomList(name: String)
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
    @Published var isMultihopEnabled: Bool
    @Published var multihopContext: MultihopContext = .exit

    @Published var exitContext = LocationContext()
    @Published var entryContext = LocationContext()
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

    private var cancellables: [Combine.AnyCancellable] = []

    var allLocations: [LocationNode] {
        exitContext.locations + exitContext.customLists + entryContext.locations + entryContext.customLists
    }

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
        isMultihopEnabled = tunnelManager.settings.tunnelMultihopState.isEnabled
        if tunnelManager.settings.tunnelMultihopState.isEnabled {
            self.multihopContext =
                if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                    .blockedState?.reason
                { .entry } else { .exit }
        }

        showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

        self.entryContext = LocationContext(
            filter: getActiveFilters(tunnelManager.settings).0,
            selectLocation: { [weak self] location in
                delegate.didSelectEntryRelayLocations(getUserSelectedRelays(location))
                self?.multihopContext = .exit
            }
        )
        self.exitContext = LocationContext(
            filter: getActiveFilters(tunnelManager.settings).1,
            selectLocation: { location in
                delegate.didSelectExitRelayLocations(getUserSelectedRelays(location))
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
                    showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

                    let (activeEntryFilter, activeExitFilter) = getActiveFilters(settings)
                    if entryContext.filter != activeEntryFilter || exitContext.filter != activeExitFilter,
                        !searchText.isEmpty
                    {
                        search(searchText: searchText)
                    }
                    entryContext.filter = activeEntryFilter
                    exitContext.filter = activeExitFilter
                    setSelection(
                        selectedExitRelays: settings.relayConstraints.exitLocations.value,
                        selectedEntryRelays: settings.relayConstraints.entryLocations.value
                    )
                    self.updateConnectedLocation(tunnelManager.tunnelStatus)
                }
            )

        $searchText
            .removeDuplicates()
            .withPreviousValue()
            .sink { [weak self] prevValue, newValue in
                if prevValue == newValue { return }
                if prevValue == nil && newValue == "" { return }
                self?.search(searchText: newValue)
                if newValue == "" {
                    self?.expandSelectedLocation()
                }
            }.store(in: &cancellables)

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        fetchLocations()
        setSelection(
            selectedExitRelays: tunnelManager.settings.relayConstraints.exitLocations.value,
            selectedEntryRelays: tunnelManager.settings.relayConstraints.entryLocations.value
        )
        updateConnectedLocation(tunnelManager.tunnelStatus)
        expandSelectedLocation()
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
        refreshCustomLists()
    }

    func showEditCustomList(name: String) {
        guard let customList = customListInteractor.fetchAll().first(where: { $0.name == name }) else {
            return
        }
        switch multihopContext {
        case .entry:
            delegate
                .showEditCustomListView(entryContext.locations, customList)
        case .exit:
            delegate
                .showEditCustomListView(exitContext.locations, customList)
        }
    }

    private func fetchLocations() {
        relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        )
        if let relaysCandidates {
            exitLocationsDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())

            exitContext.locations = exitLocationsDataSource.nodes
            if let entryRelays = relaysCandidates.entryRelays {
                entryLocationsDataSource
                    .reload(entryRelays.toLocationRelays())
                entryContext.locations =
                    entryLocationsDataSource.nodes
            }
        } else {
            entryContext.locations = []
            exitContext.locations = []
        }
        refreshCustomLists()
    }

    private func getActiveFilters(_ settings: LatestTunnelSettings) -> (
        [SelectLocationFilter],
        [SelectLocationFilter]
    ) {
        var activeEntryFilter: [SelectLocationFilter] = []
        var activeExitFilter: [SelectLocationFilter] = []

        let isMultihop = settings.tunnelMultihopState.isEnabled
        if let ownershipFilter = settings.relayConstraints.filter.value {
            switch ownershipFilter.ownership {
            case .any:
                break
            case .owned:
                activeEntryFilter.append(.owned)
                activeExitFilter.append(.owned)
            case .rented:
                activeEntryFilter.append(.rented)
                activeExitFilter.append(.rented)
            }
            if let provider = ownershipFilter.providers.value {
                activeEntryFilter.append(.provider(provider.count))
                activeExitFilter.append(.provider(provider.count))
            }
        }
        if settings.daita.isDirectOnly {
            if isMultihop {
                activeEntryFilter.append(.daita)
            } else {
                activeExitFilter.append(.daita)
            }
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            if isMultihop {
                activeEntryFilter.append(.obfuscation)
            } else {
                activeExitFilter.append(.obfuscation)
            }
        }
        return (activeEntryFilter, activeExitFilter)
    }

    private func updateConnectedLocation(_ status: TunnelStatus) {
        (exitContext.locations + exitContext.customLists)
            .forEachNode { node in
                node.isConnected = node.name == status.state.relays?.exit.hostname
            }
        (entryContext.locations + entryContext.customLists)
            .forEachNode { node in
                node.isConnected = node.name == status.state.relays?.entry?.hostname
            }
        // For some reason the view does not update the connected label without this call.
        self.objectWillChange.send()
    }

    func refreshCustomLists() {
        exitCustomListsDataSource.reload(allLocationNodes: exitContext.locations)
        entryCustomListsDataSource.reload(allLocationNodes: entryContext.locations)

        exitContext.customLists =
            exitCustomListsDataSource.nodes
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

        entryContext.customLists =
            entryCustomListsDataSource.nodes
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
    }

    private func search(searchText: String) {
        exitLocationsDataSource.search(by: searchText)
        entryLocationsDataSource.search(by: searchText)
        exitCustomListsDataSource.search(by: searchText)
        entryCustomListsDataSource.search(by: searchText)
    }

    private func getSelectedLocationNode(selectedRelays: UserSelectedRelays?, context: MultihopContext) -> LocationNode?
    {
        let allLocationsDataSource: AllLocationDataSource? =
            switch context {
            case .entry:
                entryLocationsDataSource
            case .exit:
                exitLocationsDataSource
            }

        let customListsDataSource: CustomListsDataSource? =
            switch context {
            case .entry:
                entryCustomListsDataSource
            case .exit:
                exitCustomListsDataSource
            }

        if let selectedRelays {
            // Look for a matching custom list node.
            if let customListSelection = selectedRelays.customListSelection,
                let customList = customListsDataSource?.customList(by: customListSelection.listId),
                let selectedNode = customListsDataSource?.node(by: selectedRelays, for: customList)
            {
                return selectedNode
                // Look for a matching all locations node.
            } else if let location = selectedRelays.locations.first,
                let selectedNode = allLocationsDataSource?.node(by: location)
            {
                return selectedNode
            }
        }
        return nil
    }

    private func excludeSelectedRelays(
        selectedRelays: UserSelectedRelays?,
        inContext context: MultihopContext
    ) {
        let otherAllLocation =
            switch context {
            case .entry:
                exitContext.locations
            case .exit:
                entryContext.locations
            }

        let otherCustomLists =
            switch context {
            case .entry:
                exitContext.customLists
            case .exit:
                entryContext.customLists
            }

        guard let selectedRelayLocations = selectedRelays?.locations,
            selectedRelayLocations.count == 1,
            let selectedRelayLocation = selectedRelayLocations.first
        else {
            return
        }
        let allOtherLocations = otherAllLocation + otherCustomLists
        allOtherLocations.forEachNode { node in
            let locations = Set((node.flattened + [node]).flatMap { $0.locations })
            if locations
                .contains(selectedRelayLocation) && node.activeRelayNodes.count == 1
            {
                node.isExcluded = true
                node.forEachDescendant { child in
                    child.isExcluded = true
                }
            }
        }
    }

    private func setSelection(
        selectedExitRelays: UserSelectedRelays?,
        selectedEntryRelays: UserSelectedRelays?
    ) {
        // reset all nodes
        allLocations
            .forEachNode { node in
                node.isSelected = false
                node.isExcluded = false
            }
        // set exit selection
        if let selectedExitNode = getSelectedLocationNode(
            selectedRelays: selectedExitRelays,
            context: .exit
        ) {
            selectedExitNode.isSelected = true
        }

        if isMultihopEnabled {
            // set entry selection
            if let selectedEntryNode = getSelectedLocationNode(
                selectedRelays: selectedEntryRelays,
                context: .entry
            ) {
                selectedEntryNode.isSelected = true
            }

            // exclude selected entry relays in exit lists
            excludeSelectedRelays(
                selectedRelays: selectedEntryRelays, inContext: .entry)
            // exclude selected exit relays in entry lists
            excludeSelectedRelays(
                selectedRelays: selectedExitRelays,
                inContext: .exit
            )
        }
        expandSelectedLocation()
    }

    private func expandSelectedLocation() {
        allLocations.forEachNode { node in
            if node.isSelected {
                node.forEachAncestor { $0.showsChildren = true }
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
        refreshCustomLists()
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
        refreshCustomLists()
    }

    func didFinish() {
        delegate.didFinish()
    }

    func showDaitaSettings() {
        delegate.showDaitaSettings()
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
