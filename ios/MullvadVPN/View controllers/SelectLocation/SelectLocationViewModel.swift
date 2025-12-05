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
    var isRecentsEnabled: Bool { get }
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
    func toggleRecents()
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
    @Published var isRecentsEnabled: Bool = true
    @Published var multihopContext: MultihopContext = .exit
    @Published var exitContext = LocationContext()
    @Published var entryContext = LocationContext()
    @Published var searchText: String = ""
    @Published var showDAITAInfo: Bool

    private let exitLocationsDataSource = AllLocationDataSource()
    private let entryLocationsDataSource = AllLocationDataSource()
    private let entryCustomListsDataSource: CustomListsDataSource
    private let exitCustomListsDataSource: CustomListsDataSource
    private let entryRecentsDataSource: RecentListDataSource
    private let exitRecentsDataSource: RecentListDataSource

    private let relaySelectorWrapper: RelaySelectorWrapper
    private let tunnelManager: TunnelManager
    private let customListInteractor: CustomListInteractorProtocol
    private let recentsInteractor: RecentsInteractorProtocol
    private var relaysCandidates: RelayCandidates?

    private var tunnelObserver: TunnelBlockObserver?

    private let delegate: SelectLocationDelegate

    private var cancellables = Set<Combine.AnyCancellable>()

    let entryLocationsProvider: LocationNodeResolver
    let exitLocationsProvider: LocationNodeResolver

    private var allLocations: [LocationNode] {
        exitContext.locations + exitContext.customLists + entryContext.locations + entryContext.customLists
    }

    init(
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        customListRepository: CustomListRepositoryProtocol,
        recentConnectionsRepository: RecentConnectionsRepositoryProtocol,
        delegate: SelectLocationDelegate
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.customListInteractor = CustomListInteractor(
            tunnelManager: tunnelManager,
            repository: customListRepository
        )
        self.recentsInteractor = RecentsInteractor(
            selectedEntryRelays: tunnelManager.settings.selectedEntryRelays,
            selectedExitRelays: tunnelManager.settings.selectedExitRelays,
            repository: recentConnectionsRepository)

        self.delegate = delegate
        self.entryCustomListsDataSource = CustomListsDataSource(
            repository: customListRepository
        )
        self.exitCustomListsDataSource = CustomListsDataSource(
            repository: customListRepository
        )

        self.entryRecentsDataSource = RecentListDataSource(
            entryLocationsDataSource, customListsDataSource: entryCustomListsDataSource)
        self.exitRecentsDataSource = RecentListDataSource(
            exitLocationsDataSource, customListsDataSource: exitCustomListsDataSource)

        entryLocationsProvider = LocationNodeResolver(
            providers: [
                entryRecentsDataSource,
                entryCustomListsDataSource,
                entryLocationsDataSource,
            ]
        )

        exitLocationsProvider = LocationNodeResolver(
            providers: [
                exitRecentsDataSource,
                exitCustomListsDataSource,
                exitLocationsDataSource,
            ]
        )

        showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

        // If multihop is enabled, we should check if there's a DAITA related error when opening the location
        // view. If there is, help the user by showing the entry instead of the exit view.
        isMultihopEnabled = tunnelManager.settings.tunnelMultihopState.isEnabled

        // Reactively keep `isRecentsEnabled` in sync with the interactor's enabled state.
        recentsInteractor
            .isEnabledPublisher
            .sink(receiveValue: { [weak self] isEnabled in
                guard let self else { return }
                isRecentsEnabled = isEnabled
                refreshRecents()
                updateSelections()
            })
            .store(in: &cancellables)

        if isMultihopEnabled {
            self.multihopContext =
                if case .noRelaysSatisfyingDaitaConstraints = tunnelManager.tunnelStatus.observedState
                    .blockedState?.reason
                { .entry } else { .exit }
        }

        self.entryContext = LocationContext(
            filter: SelectLocationFilter.getActiveFilters(tunnelManager.settings).0,
            selectLocation: { [weak self] location in
                guard let self else { return }
                recentsInteractor.updateSelectedLocations(location.userSelectedRelays, for: .entry)
                delegate.didSelectEntryRelayLocations(location.userSelectedRelays)
                multihopContext = .exit
            }
        )
        self.exitContext = LocationContext(
            filter: SelectLocationFilter.getActiveFilters(tunnelManager.settings).1,
            selectLocation: { [weak self] location in
                guard let self else { return }
                recentsInteractor.updateSelectedLocations(location.userSelectedRelays, for: .exit)
                delegate.didSelectExitRelayLocations(location.userSelectedRelays)
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
                    refreshRecents()
                    updateSelections()
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
                    self?.updateSelections()
                }
            }.store(in: &cancellables)

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver

        fetchLocations()
        refreshCustomLists()
        refreshRecents()
        updateSelections()
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
        refreshRecents()
        updateSelections()
        updateConnectedLocations(tunnelManager.tunnelStatus)
    }

    private func refreshRecents() {
        entryRecentsDataSource.reload(recentsInteractor.fetch(context: .entry))
        exitRecentsDataSource.reload(recentsInteractor.fetch(context: .exit))
        entryContext.recents = entryRecentsDataSource.nodes
        exitContext.recents = exitRecentsDataSource.nodes
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
    }

    private func updateConnectedLocations(_ status: TunnelStatus) {
        entryLocationsProvider.setConnectedRelay(hostname: status.state.relays?.entry?.hostname)
        exitLocationsProvider.setConnectedRelay(hostname: status.state.relays?.exit.hostname)
    }

    private func search(searchText: String) {

        exitLocationsDataSource.search(by: searchText)
        exitCustomListsDataSource.search(by: searchText)
        exitRecentsDataSource.search(by: searchText)

        entryLocationsDataSource.search(by: searchText)
        entryCustomListsDataSource.search(by: searchText)
        entryRecentsDataSource.search(by: searchText)
    }

    private func updateSelections() {
        let selectedEntryRelays = tunnelManager.settings.selectedEntryRelays
        let selectedExitRelays = tunnelManager.settings.selectedExitRelays

        entryLocationsProvider.setSelectedNode(selectedRelays: selectedEntryRelays)
        exitLocationsProvider.setSelectedNode(selectedRelays: selectedExitRelays)

        entryLocationsProvider.setExcludedNode(excludedSelection: selectedExitRelays)
        exitLocationsProvider.setExcludedNode(excludedSelection: selectedEntryRelays)

        updateConnectedLocations(tunnelManager.tunnelStatus)

        setSelectedNodeExpanded(!isRecentsEnabled)
    }

    private func setSelectedNodeExpanded(_ isExpanded: Bool) {
        exitLocationsProvider.setSelectedNodeExpanded(isExpanded)
        entryLocationsProvider.setSelectedNodeExpanded(isExpanded)
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

    func toggleRecents() {
        recentsInteractor.toggle()
    }
}

private extension LatestTunnelSettings {
    var selectedEntryRelays: UserSelectedRelays {
        relayConstraints.entryLocations.value ?? .default
    }
    var selectedExitRelays: UserSelectedRelays {
        relayConstraints.exitLocations.value ?? .default
    }
}
