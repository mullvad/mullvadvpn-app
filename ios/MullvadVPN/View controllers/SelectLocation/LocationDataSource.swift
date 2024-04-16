//
//  LocationDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

final class LocationDataSource:
    UITableViewDiffableDataSource<LocationSection, LocationCellViewModel>,
    LocationDiffableDataSourceProtocol {
    private var currentSearchString = ""
    private var dataSources: [LocationDataSourceProtocol] = []
    private var selectedItem: LocationCellViewModel?
    private var hasFilter = false
    let tableView: UITableView
    let sections: [LocationSection]

    var didSelectRelayLocations: ((UserSelectedRelays) -> Void)?
    var didTapEditCustomLists: (() -> Void)?

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

    func setRelays(_ response: REST.ServerRelaysResponse, selectedRelays: UserSelectedRelays?, filter: RelayFilter) {
        hasFilter = filter.providers != .any || filter.ownership != .any

        let allLocationsDataSource =
            dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource

        let customListsDataSource =
            dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource

        let relays = response.wireguard.relays.filter { relay in
            RelaySelector.relayMatchesFilter(relay, filter: filter)
        }

        allLocationsDataSource?.reload(response, relays: relays)
        customListsDataSource?.reload(allLocationNodes: allLocationsDataSource?.nodes ?? [], isFiltered: hasFilter)

        mapSelectedItem(from: selectedRelays)
        filterRelays(by: currentSearchString)
    }

    func filterRelays(by searchString: String) {
        currentSearchString = searchString

        let list = sections.enumerated().map { index, section in
            dataSources[index]
                .search(by: searchString)
                .flatMap { node in
                    let rootNode = RootLocationNode(children: [node])
                    return recursivelyCreateCellViewModelTree(for: rootNode, in: section, indentationLevel: 0)
                }
        }

        updateDataSnapshot(with: list, reloadExisting: !searchString.isEmpty) {
            if searchString.isEmpty {
                self.setSelectedItem(self.selectedItem, animated: false, completion: {
                    self.scrollToSelectedRelay()
                })
            } else {
                self.scrollToTop(animated: false)
            }
        }
    }

    /// Refreshes the custom list section and keeps all modifications intact (selection and expanded states).
    func refreshCustomLists(selectedRelays: UserSelectedRelays?) {
        guard let allLocationsDataSource =
            dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource,
            let customListsDataSource =
            dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource
        else {
            return
        }

        // Take a "snapshot" of the currently expanded nodes.
        let expandedNodes = customListsDataSource.nodes
            .flatMap { [$0] + $0.flattened }
            .filter { $0.showsChildren }

        // Reload data source with (possibly) updated custom lists.
        customListsDataSource.reload(allLocationNodes: allLocationsDataSource.nodes, isFiltered: hasFilter)

        // Reapply current selection.
        mapSelectedItem(from: selectedRelays)

        // Reapply current search filter.
        let searchResultNodes = dataSources[0].search(by: currentSearchString)

        // Reapply expanded status and override nodes being hidden by search filter.
        RootLocationNode(children: searchResultNodes).forEachDescendant { node in
            node.showsChildren = expandedNodes.contains(node)
            node.isHiddenFromSearch = false
        }

        // Construct node tree.
        let list = searchResultNodes.flatMap { node in
            let rootNode = RootLocationNode(children: [node])
            return recursivelyCreateCellViewModelTree(for: rootNode, in: .customLists, indentationLevel: 0)
        }

        updateDataSnapshot(with: [
            list,
            snapshot().itemIdentifiers(inSection: .allLocations),
        ], reloadExisting: true)
    }

    // Called from `LocationDiffableDataSourceProtocol`.
    func nodeShowsChildren(_ node: LocationNode) -> Bool {
        node.showsChildren
    }

    // Called from `LocationDiffableDataSourceProtocol`.
    func nodeShouldBeSelected(_ node: LocationNode) -> Bool {
        false
    }

    private func indexPathForSelectedRelay() -> IndexPath? {
        selectedItem.flatMap { indexPath(for: $0) }
    }

    private func mapSelectedItem(from selectedRelays: UserSelectedRelays?) {
        let allLocationsDataSource =
            dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource

        let customListsDataSource =
            dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource

        if let selectedRelays {
            // Look for a matching custom list node.
            if let customListSelection = selectedRelays.customListSelection,
               let customList = customListsDataSource?.customList(by: customListSelection.listId),
               let selectedNode = customListsDataSource?.node(by: selectedRelays, for: customList) {
                selectedItem = LocationCellViewModel(
                    section: .customLists,
                    node: selectedNode,
                    indentationLevel: selectedNode.hierarchyLevel
                )
                // Look for a matching all locations node.
            } else if let location = selectedRelays.locations.first,
                      let selectedNode = allLocationsDataSource?.node(by: location) {
                selectedItem = LocationCellViewModel(
                    section: .allLocations,
                    node: selectedNode,
                    indentationLevel: selectedNode.hierarchyLevel
                )
            }
        }
    }

    private func setSelectedItem(_ item: LocationCellViewModel?, animated: Bool, completion: (() -> Void)? = nil) {
        selectedItem = item
        guard let selectedItem else { return }

        let rootNode = selectedItem.node.root

        // Exit early if no changes to the node tree should be made.
        guard selectedItem.node != rootNode else {
            completion?()
            return
        }

        // Make sure we have an index path for the selected item.
        guard let indexPath = indexPath(for: LocationCellViewModel(
            section: selectedItem.section,
            node: rootNode
        )) else { return }

        // Walk tree backwards to determine which nodes should be expanded.
        selectedItem.node.forEachAncestor { node in
            node.showsChildren = true
        }

        // Construct node tree.
        let nodesToAdd = recursivelyCreateCellViewModelTree(
            for: rootNode,
            in: selectedItem.section,
            indentationLevel: 1
        )

        // Insert the new node tree below the selected item.
        var snapshotItems = snapshot().itemIdentifiers(inSection: selectedItem.section)
        snapshotItems.insert(contentsOf: nodesToAdd, at: indexPath.row + 1)

        let list = sections.enumerated().map { index, section in
            index == indexPath.section
                ? snapshotItems
                : snapshot().itemIdentifiers(inSection: section)
        }

        updateDataSnapshot(
            with: list,
            reloadExisting: true,
            animated: animated,
            completion: completion
        )
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        // swiftlint:disable:next force_cast
        let cell = super.tableView(tableView, cellForRowAt: indexPath) as! LocationCell
        cell.delegate = self
        return cell
    }
}

extension LocationDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        switch sections[section] {
        case .allLocations:
            return LocationSectionHeaderView(
                configuration: LocationSectionHeaderView.Configuration(name: LocationSection.allLocations.description)
            )
        case .customLists:
            return LocationSectionHeaderView(configuration: LocationSectionHeaderView.Configuration(
                name: LocationSection.customLists.description,
                primaryAction: UIAction(
                    handler: { [weak self] _ in
                        self?.didTapEditCustomLists?()
                    }
                )
            ))
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        nil
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        switch sections[section] {
        case .allLocations:
            return .zero
        case .customLists:
            return 24
        }
    }

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        itemIdentifier(for: indexPath)?.node.isActive ?? false
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        itemIdentifier(for: indexPath)?.indentationLevel ?? 0
    }

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        if let item = itemIdentifier(for: indexPath), item == selectedItem {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }

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

    private func scrollToSelectedRelay() {
        indexPathForSelectedRelay().flatMap {
            tableView.scrollToRow(at: $0, at: .middle, animated: false)
        }
    }
}

extension LocationDataSource: LocationCellDelegate {
    func toggleExpanding(cell: LocationCell) {
        guard let indexPath = tableView.indexPath(for: cell),
              let item = itemIdentifier(for: indexPath) else { return }

        let items = toggledItems(for: cell)

        updateDataSnapshot(with: items, reloadExisting: true, completion: {
            self.scroll(to: item, animated: true)
        })
    }

    func toggleSelecting(cell: LocationCell) {
        // No op.
    }
}
