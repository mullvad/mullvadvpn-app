//
//  LocationDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

final class LocationDataSource:
    UITableViewDiffableDataSource<LocationSection, LocationCellViewModel>,
    LocationDiffableDataSourceProtocol {
    nonisolated(unsafe) private var currentSearchString = ""
    nonisolated(unsafe) private var dataSources: [LocationDataSourceProtocol] = []
    // The selected location.
    nonisolated(unsafe) private var selectedLocation: LocationCellViewModel?
    // When multihop is enabled, this is the "inverted" selected location, ie. entry
    // if in exit mode and exit if in entry mode.
    nonisolated(unsafe) private var excludedLocation: LocationCellViewModel?
    let tableView: UITableView
    let sections: [LocationSection]

    var didSelectRelayLocations: (@Sendable (UserSelectedRelays) -> Void)?
    var didTapEditCustomLists: (@Sendable () -> Void)?

    init(
        tableView: UITableView,
        allLocations: LocationDataSourceProtocol,
        customLists: LocationDataSourceProtocol
    ) {
        self.tableView = tableView

        let sections: [LocationSection] = LocationSection.allCases
        self.sections = sections

        self.dataSources.append(contentsOf: [customLists, allLocations])

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            let cell = tableView.dequeueReusableView(
                withIdentifier: sections[indexPath.section],
                for: indexPath
            ) as! LocationCell // swiftlint:disable:this force_cast
            cell.configure(item: itemIdentifier, behavior: .select)
            return cell
        }

        tableView.delegate = self
        tableView.registerReusableViews(from: LocationSection.self)
        defaultRowAnimation = .fade
    }

    func setRelays(_ relaysWithLocation: LocationRelays, selectedRelays: RelaySelection) {
        Task { @MainActor in
            guard let allLocationsDataSource = dataSources
                .first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource,
                let customListsDataSource = dataSources
                .first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource else { return }
            allLocationsDataSource.reload(relaysWithLocation)
            customListsDataSource.reload(allLocationNodes: allLocationsDataSource.nodes)
            Task { @MainActor in
                setSelectedRelays(selectedRelays)
                filterRelays(by: currentSearchString)
            }
        }
    }

    func filterRelays(by searchString: String) {
        Task { @MainActor in
            currentSearchString = searchString

            let list = sections.enumerated().map { index, section in
                self.dataSources[index]
                    .search(by: searchString)
                    .flatMap { node in
                        let rootNode = RootLocationNode(children: [node])
                        return self.recursivelyCreateCellViewModelTree(
                            for: rootNode,
                            in: section,
                            indentationLevel: 0
                        )
                    }
            }

            DispatchQueue.main.async {
                self.reloadDataSnapshot(with: list) {
                    if searchString.isEmpty, let selectedLocation = self.selectedLocation {
                        self.updateSelection(selectedLocation: selectedLocation, completion: {
                            self.scrollToSelectedRelay()
                        })
                    } else {
                        self.scrollToTop(animated: false)
                    }
                }
            }
        }
    }

    /// Refreshes the custom list section and keeps all modifications intact (selection and expanded states).
    func refreshCustomLists() {
        Task { @MainActor in
            guard let allLocationsDataSource =
                dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource,
                let customListsDataSource =
                dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource
            else {
                return
            }

            // Reload data source with (possibly) updated custom lists.
            customListsDataSource.reload(allLocationNodes: allLocationsDataSource.nodes)
            self.filterRelays(by: currentSearchString)
        }
    }

    func setSelectedRelays(_ selectedRelays: RelaySelection) {
        Task { @MainActor in
            guard let _selectedLocation = mapSelection(from: selectedRelays.selected) else { return }
            selectedLocation = _selectedLocation
            excludedLocation = mapSelection(from: selectedRelays.excluded)
            excludedLocation?.excludedRelayTitle = selectedRelays.excludedTitle
            self.updateSelection(selectedLocation: _selectedLocation, completion: {
                self.scrollToSelectedRelay()
            })
        }
    }

    // MARK: - Private functions

    private func scrollToSelectedRelay() {
        indexPathForSelectedRelay()
            .flatMap {
                tableView.scrollToRow(at: $0, at: .middle, animated: false)
            }
    }

    private func indexPathForSelectedRelay() -> IndexPath? {
        selectedLocation.flatMap { indexPath(for: $0) }
    }

    private func mapSelection(from selectedRelays: UserSelectedRelays?) -> LocationCellViewModel? {
        let allLocationsDataSource =
            dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource

        let customListsDataSource =
            dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource

        if let selectedRelays {
            // Look for a matching custom list node.
            if let customListSelection = selectedRelays.customListSelection,
               let customList = customListsDataSource?.customList(by: customListSelection.listId),
               let selectedNode = customListsDataSource?.node(by: selectedRelays, for: customList) {
                return LocationCellViewModel(
                    section: .customLists,
                    node: selectedNode,
                    indentationLevel: selectedNode.hierarchyLevel
                )
                // Look for a matching all locations node.
            } else if let location = selectedRelays.locations.first,
                      let selectedNode = allLocationsDataSource?.node(by: location) {
                return LocationCellViewModel(
                    section: .allLocations,
                    node: selectedNode,
                    indentationLevel: selectedNode.hierarchyLevel
                )
            }
        }

        return nil
    }

    private func updateSelection(selectedLocation: LocationCellViewModel, completion: (() -> Void)? = nil) {
        let rootNode = selectedLocation.node.root
        var snapshot = snapshot()

        // Exit early if no changes to the node tree should be made.
        guard selectedLocation.node != rootNode else {
            // Apply the updated snapshot
            DispatchQueue.main.async {
                self.applySnapshotUsingReloadData(snapshot, completion: completion)
            }
            return
        }

        // Make sure we have an index path for the selected item.
        guard let indexPath = indexPath(for: LocationCellViewModel(
            section: selectedLocation.section,
            node: rootNode
        )) else { return }

        // Walk tree backwards to determine which nodes should be expanded.
        selectedLocation.node.forEachAncestor { node in
            node.showsChildren = true
        }

        // Construct node tree.
        let nodesToAdd = recursivelyCreateCellViewModelTree(
            for: rootNode,
            in: selectedLocation.section,
            indentationLevel: 1
        )

        let existingItems = snapshot.itemIdentifiers(inSection: selectedLocation.section)
        snapshot.deleteItems(nodesToAdd)
        snapshot.insertItems(nodesToAdd, afterItem: existingItems[indexPath.row])

        // Apply the updated snapshot
        DispatchQueue.main.async {
            self.applySnapshotUsingReloadData(snapshot, completion: completion)
        }
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = super.tableView(tableView, cellForRowAt: indexPath)
        guard let cell = cell as? LocationCell, let item = itemIdentifier(for: indexPath) else {
            return cell
        }

        cell.delegate = self

        if item.shouldExcludeLocation(excludedLocation) {
            // Only host locations should have an excluded title. Since custom list nodes contain
            // all locations of all child nodes, its first location could possibly be a host.
            // Therefore we need to check for that as well.
            if case .hostname = item.node.locations.first, !(item.node is CustomListLocationNode) {
                cell.setExcluded(relayTitle: excludedLocation?.excludedRelayTitle)
            } else {
                cell.setExcluded()
            }
        }

        return cell
    }
}

// MARK: - Called from LocationDiffableDataSourceProtocol

extension LocationDataSource {
    func nodeShowsChildren(_ node: LocationNode) -> Bool {
        node.showsChildren
    }

    func nodeShouldBeSelected(_ node: LocationNode) -> Bool {
        false // N/A
    }
}

extension LocationDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        switch sections[section] {
        case .allLocations:
            return LocationSectionHeaderFooterView(
                configuration: LocationSectionHeaderFooterView.Configuration(
                    name: LocationSection.allLocations.header,
                    style: LocationSectionHeaderFooterView.Style(
                        font: .preferredFont(forTextStyle: .body, weight: .semibold),
                        textColor: .primaryTextColor,
                        backgroundColor: .primaryColor
                    )
                )
            )
        case .customLists:
            return LocationSectionHeaderFooterView(configuration: LocationSectionHeaderFooterView.Configuration(
                name: LocationSection.customLists.header,
                style: LocationSectionHeaderFooterView.Style(
                    font: .preferredFont(forTextStyle: .body, weight: .semibold),
                    textColor: .primaryTextColor,
                    backgroundColor: .primaryColor
                ),
                primaryAction: UIAction(
                    handler: { [weak self] _ in
                        self?.didTapEditCustomLists?()
                    }
                )
            ))
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        switch sections[section] {
        case .allLocations:
            return LocationSectionHeaderFooterView(configuration: LocationSectionHeaderFooterView.Configuration(
                name: LocationSection.allLocations.footer,
                style: LocationSectionHeaderFooterView.Style(
                    font: .preferredFont(forTextStyle: .body, weight: .regular),
                    textColor: .secondaryTextColor,
                    backgroundColor: .clear
                )
            ))
        case .customLists:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        switch sections[section] {
        case .allLocations:
            return dataSources[section].nodes.isEmpty ? UITableView.automaticDimension : .zero
        case .customLists:
            return 24
        }
    }

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        guard let item = itemIdentifier(for: indexPath) else { return false }
        return !item.shouldExcludeLocation(excludedLocation) && item.node.isActive
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        itemIdentifier(for: indexPath)?.indentationLevel ?? 0
    }

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        if let item = itemIdentifier(for: indexPath) {
            cell.setSelected(item == selectedLocation, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        if let indexPath = indexPathForSelectedRelay() {
            tableView.deselectRow(at: indexPath, animated: false)
        }
        return indexPath
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }
        selectedLocation = item
        var customListSelection: UserSelectedRelays.CustomListSelection?
        if let topmostNode = item.node.root as? CustomListLocationNode {
            customListSelection = UserSelectedRelays.CustomListSelection(
                listId: topmostNode.customList.id,
                isList: topmostNode == item.node
            )
        }

        let relayLocations = UserSelectedRelays(
            locations: item.node.locations,
            customListSelection: customListSelection
        )
        didSelectRelayLocations?(relayLocations)
    }

    private func scrollToTop(animated: Bool) {
        tableView.setContentOffset(.zero, animated: animated)
    }
}

extension LocationDataSource: @preconcurrency LocationCellDelegate {
    func toggleExpanding(cell: LocationCell) {
        guard let indexPath = tableView.indexPath(for: cell),
              let item = itemIdentifier(for: indexPath) else { return }
        toggleItems(for: cell) {
            self.scroll(to: item, animated: true)
        }
    }

    func toggleSelecting(cell: LocationCell) {
        // No op.
    }
}
