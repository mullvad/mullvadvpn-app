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
    func customListsChanged()
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

    private let allLocationsDataSource = AllLocationDataSource()
    private let exitLocationsDataSource = AllLocationDataSource()
    private let entryLocationsDataSource = AllLocationDataSource()
    private let entryCustomListsDataSource: CustomListsDataSource
    private let exitCustomListsDataSource: CustomListsDataSource

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListInteractor: CustomListInteractorProtocol
    private var relaysCandidates: RelayCandidates?

    private var tunnelObserver: TunnelBlockObserver?

    private let delegate: SelectLocationDelegate

    private var cancellables: [Combine.AnyCancellable] = []

    private var allLocations: [LocationNode] {
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

        showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

        // If multihop is enabled, we should check if there's a DAITA related error when opening the location
        // view. If there is, help the user by showing the entry instead of the exit view.
        isMultihopEnabled = tunnelManager.settings.tunnelMultihopState.isEnabled
        if isMultihopEnabled {
            self.multihopContext =
                if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                    .blockedState?.reason
                { .entry } else { .exit }
        }

        self.entryContext = LocationContext(
            filter: SelectLocationFilter.getActiveFilters(tunnelManager.settings).0,
            selectLocation: { [weak self] location in
                delegate
                    .didSelectEntryRelayLocations(location.userSelectedRelays)
                self?.multihopContext = .exit
            }
        )
        self.exitContext = LocationContext(
            filter: SelectLocationFilter.getActiveFilters(tunnelManager.settings).1,
            selectLocation: { location in
                delegate
                    .didSelectExitRelayLocations(location.userSelectedRelays)
            }
        )
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    self?.updateConnectedLocations(status)
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    fetchLocations()
                    refreshCustomLists()
                    updateSelections(
                        selectedExitRelays: settings.relayConstraints.exitLocations.value,
                        selectedEntryRelays: settings.relayConstraints.entryLocations.value
                    )
                    updateConnectedLocations(tunnelManager.tunnelStatus)
                    if !searchText.isEmpty {
                        search(searchText: searchText)
                    }

                    showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

                    let (activeEntryFilter, activeExitFilter) = SelectLocationFilter.getActiveFilters(
                        settings
                    )
                    entryContext.filter = activeEntryFilter
                    exitContext.filter = activeExitFilter

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
        refreshCustomLists()
        updateSelections(
            selectedExitRelays: tunnelManager.settings.relayConstraints.exitLocations.value,
            selectedEntryRelays: tunnelManager.settings.relayConstraints.entryLocations.value
        )
        updateConnectedLocations(tunnelManager.tunnelStatus)
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
        customListsChanged()
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

    func addLocationToCustomList(location: LocationNode, customListName: String) {
        try? customListInteractor
            .addLocationToCustomList(
                relayLocations: location.locations,
                customListName: customListName
            )
        customListsChanged()
    }

    func removeLocationFromCustomList(
        location: LocationNode,
        customListName: String
    ) {
        try? customListInteractor
            .removeLocationFromCustomList(
                relayLocations: location.locations,
                customListName: customListName
            )
        customListsChanged()
    }

    func customListsChanged() {
        refreshCustomLists()
        updateSelections(
            selectedExitRelays: tunnelManager.settings.relayConstraints.exitLocations.value,
            selectedEntryRelays: tunnelManager.settings.relayConstraints.entryLocations.value
        )
        updateConnectedLocations(tunnelManager.tunnelStatus)
    }

    private func refreshCustomLists() {
        exitCustomListsDataSource.reload(allLocationNodes: exitContext.locations)
        entryCustomListsDataSource.reload(allLocationNodes: entryContext.locations)

        exitContext.customLists = exitCustomListsDataSource.nodes
        entryContext.customLists = entryCustomListsDataSource.nodes
    }

    private func fetchLocations() {
        relaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: tunnelManager.settings
        )
        if let allRelaysCandidates = try? relaySelectorWrapper.findCandidates(
            tunnelSettings: .init(
                tunnelMultihopState: tunnelManager.settings.tunnelMultihopState
            )
        ) {
            entryContext.totalRelayCount = allRelaysCandidates.entryRelays?.count ?? 0
            exitContext.totalRelayCount = allRelaysCandidates.exitRelays.count
        } else {
            entryContext.totalRelayCount = 0
            exitContext.totalRelayCount = 0
        }
        if let relaysCandidates {
            exitLocationsDataSource
                .reload(relaysCandidates.exitRelays.toLocationRelays())
            exitContext.locations = exitLocationsDataSource.nodes
            exitContext.availableRelayCount = relaysCandidates.exitRelays.count

            if let entryRelays = relaysCandidates.entryRelays {
                entryLocationsDataSource
                    .reload(entryRelays.toLocationRelays())
                entryContext.locations =
                    entryLocationsDataSource.nodes
                entryContext.availableRelayCount = relaysCandidates.entryRelays?.count ?? 0
            }
        } else {
            entryContext.locations = []
            exitContext.locations = []
        }
    }

    private func updateConnectedLocations(_ status: TunnelStatus) {
        exitLocationsDataSource
            .setConnectedRelay(hostname: status.state.relays?.exit.hostname)
        exitCustomListsDataSource
            .setConnectedRelay(hostname: status.state.relays?.exit.hostname)
        entryLocationsDataSource
            .setConnectedRelay(hostname: status.state.relays?.entry?.hostname)
        entryCustomListsDataSource
            .setConnectedRelay(hostname: status.state.relays?.entry?.hostname)
    }

    private func search(searchText: String) {
        exitLocationsDataSource
            .search(by: searchText)
        exitCustomListsDataSource
            .search(by: searchText)
        entryLocationsDataSource
            .search(by: searchText)
        entryCustomListsDataSource
            .search(by: searchText)
    }

    private func updateSelections(
        selectedExitRelays: UserSelectedRelays?,
        selectedEntryRelays: UserSelectedRelays?
    ) {
        // set exit selection
        exitLocationsDataSource
            .setSelectedNode(selectedRelays: selectedExitRelays)
        exitCustomListsDataSource
            .setSelectedNode(selectedRelays: selectedExitRelays)

        if isMultihopEnabled {
            // set entry selection
            entryLocationsDataSource
                .setSelectedNode(selectedRelays: selectedEntryRelays)
            entryCustomListsDataSource
                .setSelectedNode(selectedRelays: selectedEntryRelays)

            // exclude selected entry relays in exit lists
            exitLocationsDataSource
                .setExcludedNode(excludedSelection: selectedEntryRelays)
            exitCustomListsDataSource
                .setExcludedNode(excludedSelection: selectedEntryRelays)

            // exclude selected exit relays in entry lists
            entryLocationsDataSource
                .setExcludedNode(excludedSelection: selectedExitRelays)
            entryCustomListsDataSource
                .setExcludedNode(excludedSelection: selectedExitRelays)
        }
    }

    private func expandSelectedLocation() {
        exitLocationsDataSource
            .expandSelection()
        exitCustomListsDataSource
            .expandSelection()
        entryLocationsDataSource
            .expandSelection()
        entryCustomListsDataSource
            .expandSelection()
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
