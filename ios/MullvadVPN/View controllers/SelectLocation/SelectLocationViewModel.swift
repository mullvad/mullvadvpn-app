import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import SwiftUI

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
    func toggleMultihop()
    func evaluateMultihopSetting(_ state: MultihopState) -> Bool
    func toggleRecents()
    func manuallyFetchRelayList()
}

struct SelectLocationDelegate {
    let showDaitaSettings: () -> Void
    let showObfuscationSettings: () -> Void
    let showIpVersionSettings: () -> Void
    let showFilterView: (MultihopContext) -> Void
    let showEditCustomListView: ([LocationNode], CustomList?) -> Void
    let showAddCustomListView: ([LocationNode]) -> Void
    let didSelectExitRelayLocations: (RelayConstraint<UserSelectedRelays>) -> Void
    let didSelectEntryRelayLocations: (RelayConstraint<UserSelectedRelays>) -> Void
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
    private let relayCacheTracker: RelayCacheTrackerProtocol
    private let tunnelManager: TunnelManager
    private let customListInteractor: CustomListInteractorProtocol
    private let recentsInteractor: RecentsInteractorProtocol
    private var relaysCandidates: RelayCandidates?

    private var tunnelObserver: TunnelBlockObserver?

    private let delegate: SelectLocationDelegate

    private var cancellables = Set<Combine.AnyCancellable>()

    private var allLocations: [LocationNode] {
        exitContext.locations + exitContext.customLists + entryContext.locations + entryContext.customLists
    }

    init(
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        relayCacheTracker: RelayCacheTrackerProtocol,
        customListRepository: CustomListRepositoryProtocol,
        recentConnectionsRepository: RecentConnectionsRepositoryProtocol,
        delegate: SelectLocationDelegate
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.relayCacheTracker = relayCacheTracker
        self.customListInteractor = CustomListInteractor(
            tunnelManager: tunnelManager,
            repository: customListRepository
        )
        self.recentsInteractor = RecentsInteractor(
            selectedEntryConstraint: tunnelManager.settings.relayConstraints.entryLocations,
            selectedExitConstraint: tunnelManager.settings.relayConstraints.exitLocations,
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

        showDAITAInfo = tunnelManager.settings.daita.isAutomaticRouting

        // If multihop is enabled, we should check if there's a DAITA related error when opening the location
        // view. If there is, help the user by showing the entry instead of the exit view.
        isMultihopEnabled = tunnelManager.settings.tunnelMultihopState.isUserSelected

        // Reactively keep `isRecentsEnabled` in sync with the interactor's enabled state.
        recentsInteractor
            .isEnabledPublisher
            .sink(receiveValue: { [weak self] isEnabled in
                guard let self else { return }
                reloadAllDataSources()
                updateSelections()
                isRecentsEnabled = isEnabled
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

                let constraint: RelayConstraint<UserSelectedRelays> =
                    if location is AutomaticLocationNode {
                        .any
                    } else {
                        .only(location.userSelectedRelays)
                    }

                recentsInteractor.updateSelectedLocations(constraint, for: .entry)
                delegate.didSelectEntryRelayLocations(constraint)

                multihopContext = .exit
            }
        )
        self.exitContext = LocationContext(
            filter: SelectLocationFilter.getActiveFilters(tunnelManager.settings).1,
            selectLocation: { [weak self] location in
                guard let self else { return }

                let constraint = RelayConstraint.only(location.userSelectedRelays)

                recentsInteractor.updateSelectedLocations(constraint, for: .exit)
                delegate.didSelectExitRelayLocations(constraint)
            }
        )
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, status in
                    self?.updateConnectedLocations(status)
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self else { return }
                    isMultihopEnabled = settings.tunnelMultihopState.isUserSelected
                    if !isMultihopEnabled {
                        multihopContext = .exit
                    }
                    reloadAllDataSources()
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
        reloadAllDataSources()
        updateSelections()
        updateConnectedLocations(tunnelManager.tunnelStatus)
    }

    deinit {
        guard let tunnelObserver else { return }
        tunnelManager.removeObserver(tunnelObserver)
    }

    func toggleMultihop() {
        let multihopState: MultihopState = isMultihopEnabled ? .never : .always
        tunnelManager.updateSettings([.multihop(multihopState)])
    }

    func evaluateMultihopSetting(_ state: MultihopState) -> Bool {
        var tunnelSettings = tunnelManager.settings
        tunnelSettings.tunnelMultihopState = state

        if !tunnelSettings.automaticMultihopIsEnabled {
            let relays = try? tunnelManager.selectRelays(tunnelSettings: tunnelSettings)
            return relays != nil
        }

        return true
    }

    func onFilterTapped(_ filter: SelectLocationFilter) {
        switch filter {
        case .owned, .rented, .provider:
            delegate.showFilterView(multihopContext)
        case .daita:
            delegate.showDaitaSettings()
        case .obfuscation:
            delegate.showObfuscationSettings()
        case .ipv6:
            delegate.showIpVersionSettings()
        }
    }

    func onFilterRemoved(_ filter: SelectLocationFilter) {
        var relayConstraints = tunnelManager.settings.relayConstraints

        switch filter {
        case .owned, .rented:
            guard var filter = relayConstraints.filterConstraint(for: multihopContext).value else { return }
            filter.ownership = .any
            relayConstraints.setFilterConstraint(.only(filter), for: multihopContext)
            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
        case .provider:
            guard var filter = relayConstraints.filterConstraint(for: multihopContext).value else { return }
            filter.providers = .any
            relayConstraints.setFilterConstraint(.only(filter), for: multihopContext)
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
        recentsInteractor.cleanup(customList.id)
        customListsChanged()
    }

    func showEditCustomList(name: String) {
        guard let customList = customListInteractor.fetchAll().first(where: { $0.name == name }) else {
            return
        }
        switch multihopContext {
        case .entry:
            delegate
                .showEditCustomListView(entryContext.customListAvailableLocations, customList)
        case .exit:
            delegate
                .showEditCustomListView(exitContext.customListAvailableLocations, customList)
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

    private func reloadAllDataSources() {
        fetchLocations()
        refreshCustomLists()
        refreshRecents()
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
            exitContext.customListAvailableLocations = exitLocationsDataSource.nodes
            exitContext.availableRelayCount = relaysCandidates.exitRelays.count

            if let entryRelays = relaysCandidates.entryRelays {
                entryLocationsDataSource.reload(entryRelays.toLocationRelays())

                if tunnelManager.settings.tunnelMultihopState.isUserSelected {
                    entryLocationsDataSource.addAutomaticLocationNode()
                }

                entryContext.locations = entryLocationsDataSource.nodes
                entryContext.customListAvailableLocations = entryLocationsDataSource.nodes
                entryContext.availableRelayCount = entryRelays.count
            }
        } else {
            entryContext.locations = []
            exitContext.locations = []
        }
    }

    private func updateConnectedLocations(_ status: TunnelStatus) {
        let relayConstraints = tunnelManager.settings.relayConstraints
        let selectedRelays = status.state.relays

        ([
            entryCustomListsDataSource,
            entryLocationsDataSource,
            entryRecentsDataSource,
        ] as [LocationDataSourceProtocol]).forEach {
            $0.setConnectedRelay(
                relayConstraint: relayConstraints.entryLocations,
                selectedRelay: selectedRelays?.entry
            )
        }

        ([
            exitCustomListsDataSource,
            exitLocationsDataSource,
            exitRecentsDataSource,
        ] as [LocationDataSourceProtocol]).forEach {
            $0.setConnectedRelay(
                relayConstraint: relayConstraints.exitLocations,
                selectedRelay: selectedRelays?.exit
            )
        }
    }

    private func search(searchText: String) {
        exitContext.locations = exitLocationsDataSource.search(by: searchText)
        exitContext.customLists = exitCustomListsDataSource.search(by: searchText)
        entryContext.locations = entryLocationsDataSource.search(by: searchText)
        entryContext.customLists = entryCustomListsDataSource.search(by: searchText)
    }

    private func updateSelections() {
        let selectedEntryConstraint = tunnelManager.settings.relayConstraints.entryLocations
        let selectedExitConstraint = tunnelManager.settings.relayConstraints.exitLocations

        let updateRecentsDataSources:
            (
                LocationDataSourceProtocol,
                RelayConstraint<UserSelectedRelays>
            ) -> Void = { dataSource, selected in
                dataSource.setSelectedNode(constraint: selected)
            }

        let updateLocationsDataSources:
            (
                [LocationDataSourceProtocol],
                RelayConstraint<UserSelectedRelays>,
                RelayConstraint<UserSelectedRelays>
            ) -> Void = { dataSources, selected, excluded in
                dataSources.forEach {
                    $0.setSelectedNode(constraint: selected)
                    $0.expandSelection()

                    if self.isMultihopEnabled {
                        $0.setExcludedNode(constraint: excluded)
                    }
                }
            }

        updateLocationsDataSources(
            [entryCustomListsDataSource, entryLocationsDataSource], selectedEntryConstraint, selectedExitConstraint)
        updateLocationsDataSources(
            [exitCustomListsDataSource, exitLocationsDataSource], selectedExitConstraint, selectedEntryConstraint)

        updateRecentsDataSources(entryRecentsDataSource, selectedEntryConstraint)
        updateRecentsDataSources(exitRecentsDataSource, selectedExitConstraint)

        exitContext.selectedLocation =
            [exitRecentsDataSource, exitCustomListsDataSource, exitLocationsDataSource].firstSelectedNode
        entryContext.selectedLocation =
            [entryRecentsDataSource, entryCustomListsDataSource, entryLocationsDataSource].firstSelectedNode
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
        delegate.showFilterView(multihopContext)
    }

    func toggleRecents() {
        recentsInteractor.toggle()
    }

    func manuallyFetchRelayList() {
        _ = relayCacheTracker.fetchRelays { [weak self] _ in
            guard let self else { return }
            reloadAllDataSources()
            updateSelections()
            updateConnectedLocations(tunnelManager.tunnelStatus)
        }
    }
}
