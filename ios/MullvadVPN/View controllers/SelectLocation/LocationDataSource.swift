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

final class LocationDataSource: UITableViewDiffableDataSource<LocationSection, LocationCellViewModel> {
    private var currentSearchString = ""
    private let tableView: UITableView
    private let locationCellFactory: LocationCellFactory
    private var dataSources: [LocationDataSourceProtocol] = []
    private var selectedItem: LocationCellViewModel?

    var didSelectRelayLocations: (([RelayLocation], UUID?) -> Void)?

    init(
        tableView: UITableView,
        allLocations: LocationDataSourceProtocol,
        customLists: LocationDataSourceProtocol
    ) {
        self.tableView = tableView
        self.dataSources.append(customLists)
        self.dataSources.append(allLocations)

        let locationCellFactory = LocationCellFactory(
            tableView: tableView,
            reuseIdentifier: LocationSection.Cell.locationCell.reuseIdentifier
        )
        self.locationCellFactory = locationCellFactory

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            locationCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.delegate = self
        locationCellFactory.delegate = self

        defaultRowAnimation = .fade
        registerClasses()
    }

    func setRelays(_ response: REST.ServerRelaysResponse, selectedLocations: RelayLocations?, filter: RelayFilter) {
        guard let customListsDataSource =
            dataSources.first(where: { $0 is CustomListsDataSource }) as? CustomListsDataSource,
            let allLocationsDataSource =
            dataSources.first(where: { $0 is AllLocationDataSource }) as? AllLocationDataSource
        else { return }

        let relays = response.wireguard.relays.filter { relay in
            RelaySelector.relayMatchesFilter(relay, filter: filter)
        }

        allLocationsDataSource.reload(response, relays: relays)
        customListsDataSource.reload(allLocationNodes: allLocationsDataSource.nodes)

        if let selectedLocations {
            // Look for a matching custom list node.
            if let customListId = selectedLocations.customListId,
               let customList = customListsDataSource.customList(by: customListId),
               let selectedNode = customListsDataSource.node(by: selectedLocations.locations, for: customList) {
                selectedItem = LocationCellViewModel(section: .customLists, node: selectedNode)
                // Look for a matching all locations node.
            } else if let location = selectedLocations.locations.first,
                      let selectedNode = allLocationsDataSource.node(by: location) {
                selectedItem = LocationCellViewModel(section: .allLocations, node: selectedNode)
            }
        }

        filterRelays(by: currentSearchString)
    }

    func indexPathForSelectedRelay() -> IndexPath? {
        selectedItem.flatMap { indexPath(for: $0) }
    }

    func filterRelays(by searchString: String) {
        currentSearchString = searchString

        let list = LocationSection.allCases.enumerated().map { index, section in
            dataSources[index]
                .search(by: searchString)
                .map { LocationCellViewModel(section: section, node: $0) }
        }

        updateDataSnapshot(with: list, reloadExisting: !searchString.isEmpty)

        if searchString.isEmpty {
            setSelectedItem(selectedItem, animated: false, completion: {
                self.scrollToSelectedRelay()
            })
        } else {
            scrollToTop(animated: false)
        }
    }

    private func updateDataSnapshot(
        with list: [[LocationCellViewModel]],
        reloadExisting: Bool = false,
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<LocationSection, LocationCellViewModel>()
        let sections = LocationSection.allCases

        snapshot.appendSections(sections)
        for (index, section) in sections.enumerated() {
            snapshot.appendItems(list[index], toSection: section)
        }

        if reloadExisting {
            snapshot.reloadSections(LocationSection.allCases)
        }

        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func registerClasses() {
        LocationSection.allCases.forEach {
            tableView.register(
                $0.cell.reusableViewClass,
                forCellReuseIdentifier: $0.cell.reuseIdentifier
            )
        }
    }

    private func setSelectedItem(
        _ item: LocationCellViewModel?,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        selectedItem = item
        guard let selectedItem else { return }

        let topmostAncestor = selectedItem.node.topmostAncestor
        guard selectedItem.node != topmostAncestor else { return }

        var snapshotItems = snapshot().itemIdentifiers(inSection: selectedItem.section)

        selectedItem.node.forEachAncestor { node in
            node.showsChildren = true
        }

        guard let indexPath = indexPath(for: LocationCellViewModel(
            section: selectedItem.section,
            node: topmostAncestor
        )) else { return }

        let nodesToAdd = recursivelyCreateCellViewModels(for: topmostAncestor, in: selectedItem.section)
        snapshotItems.insert(contentsOf: nodesToAdd, at: indexPath.row + 1)

        var list: [[LocationCellViewModel]] = Array(repeating: [], count: dataSources.count)
        for index in 0 ..< list.count {
            list[index] = (index == indexPath.section)
                ? snapshotItems
                : snapshot().itemIdentifiers(inSection: LocationSection.allCases[index])
        }

        updateDataSnapshot(
            with: list,
            reloadExisting: true,
            animated: animated,
            completion: completion
        )
    }

    private func recursivelyCreateCellViewModels(
        for node: LocationNode,
        in section: LocationSection
    ) -> [LocationCellViewModel] {
        var viewModels = [LocationCellViewModel]()

        node.children.forEach {
            $0.indentationLevel = node.indentationLevel + 1
            viewModels.append(LocationCellViewModel(section: section, node: $0))

            if $0.showsChildren {
                viewModels.append(contentsOf: recursivelyCreateCellViewModels(for: $0, in: section))
            }
        }

        return viewModels
    }
}

extension LocationDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        nil
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        let section = snapshot().sectionIdentifiers[section]

        switch section {
        case .customLists:
            return 24
        case .allLocations:
            return 0
        }
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        guard let item = itemIdentifier(for: indexPath) else { return 0 }
        return item.node.indentationLevel
    }

    func tableView(
        _ tableView: UITableView,
        willDisplay cell: UITableViewCell,
        forRowAt indexPath: IndexPath
    ) {
        if let item = itemIdentifier(for: indexPath),
           item == selectedItem {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard
            let item = itemIdentifier(for: indexPath),
            item.node != selectedItem?.node
        else { return }

        let topmostNode = item.node.topmostAncestor as? LocationListNode
        didSelectRelayLocations?(item.node.locations, topmostNode?.customList.id)

        selectedItem = item

        indexPathForSelectedRelay().flatMap {
            let cell = tableView.cellForRow(at: $0)
            cell?.setSelected(false, animated: false)
        }
    }
}

extension LocationDataSource: LocationCellEventHandler {
    func toggleCell(for item: LocationCellViewModel) {
        guard let indexPath = indexPath(for: item),
              let cell = tableView.cellForRow(at: indexPath)
        else { return }

        let isExpanded = item.node.showsChildren
        let section = LocationSection.allCases[indexPath.section]

        item.node.showsChildren = !isExpanded
        locationCellFactory.configureCell(
            cell,
            item: LocationCellViewModel(section: section, node: item.node),
            indexPath: indexPath
        )

        var locationList = snapshot().itemIdentifiers(inSection: section)

        if !isExpanded {
            locationList.addSubNodes(from: item.node, at: indexPath)
        } else {
            locationList.recursivelyRemoveSubNodes(from: item.node)
        }

        var list: [[LocationCellViewModel]] = Array(repeating: [], count: dataSources.count)
        for index in 0 ..< list.count {
            list[index] = (index == indexPath.section)
                ? locationList
                : snapshot().itemIdentifiers(inSection: LocationSection.allCases[index])
        }

        updateDataSnapshot(with: list, completion: {
            self.scroll(to: item, animated: true)
        })
    }
}

extension LocationDataSource {
    private func scroll(to item: LocationCellViewModel, animated: Bool) {
        guard
            let visibleIndexPaths = tableView.indexPathsForVisibleRows,
            let indexPath = indexPath(for: item)
        else { return }

        if item.node.children.count > visibleIndexPaths.count {
            tableView.scrollToRow(at: indexPath, at: .top, animated: animated)
        } else {
            if let last = item.node.children.last {
                if let lastInsertedIndexPath = self.indexPath(for: LocationCellViewModel(
                    section: LocationSection.allCases[indexPath.section],
                    node: last
                )),
                    let lastVisibleIndexPath = visibleIndexPaths.last,
                    lastInsertedIndexPath >= lastVisibleIndexPath {
                    tableView.scrollToRow(at: lastInsertedIndexPath, at: .bottom, animated: animated)
                }
            }
        }
    }

    private func scrollToTop(animated: Bool) {
        tableView.setContentOffset(.zero, animated: animated)
    }

    private func scrollToSelectedRelay() {
        if let selectedItem {
            scroll(to: selectedItem, animated: false)
        }
//        indexPathForSelectedRelay().flatMap {
//            tableView.scrollToRow(at: $0, at: .middle, animated: false)
//        }
    }
}

private extension [LocationCellViewModel] {
    mutating func addSubNodes(from node: LocationNode, at indexPath: IndexPath) {
        let section = LocationSection.allCases[indexPath.section]
        let row = indexPath.row + 1

        let locations = node.children.map {
            $0.indentationLevel = node.indentationLevel + 1
            return LocationCellViewModel(section: section, node: $0)
        }

        if row < count {
            insert(contentsOf: locations, at: row)
        } else {
            append(contentsOf: locations)
        }
    }

    mutating func recursivelyRemoveSubNodes(from node: LocationNode) {
        for node in node.children {
            removeAll(where: { node == $0.node })
            recursivelyRemoveSubNodes(from: node)
        }
    }
}
