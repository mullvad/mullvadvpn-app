//
//  LocationDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes
import UIKit

final class LocationDataSource: UITableViewDiffableDataSource<SelectLocationSection, LocationCellViewModel> {
    private var currentSearchString = ""
    private let tableView: UITableView
    private let locationCellFactory: LocationCellFactory
    private var dataSources: [LocationDataSourceProtocol] = []

    var selectedRelayLocation: LocationCellViewModel?
    var didSelectRelayLocation: ((RelayLocation) -> Void)?

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
            reuseIdentifier: SelectLocationSection.Cell.locationCell.reuseIdentifier
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

    func setRelays(_ response: REST.ServerRelaysResponse, filter: RelayFilter) {
        let relays = response.wireguard.relays.filter { relay in
            return RelaySelector.relayMatchesFilter(relay, filter: filter)
        }
        var list: [[LocationCellViewModel]] = []
        for section in 0 ..< dataSources.count {
            list.append(
                dataSources[section]
                    .reload(response, relays: relays)
                    .map { LocationCellViewModel(group: SelectLocationSection.allCases[section], location: $0) }
            )
        }
        filterRelays(by: currentSearchString)
    }

    func indexPathForSelectedRelay() -> IndexPath? {
        selectedRelayLocation.flatMap {
            indexPath(for: $0)
        }
    }

    func filterRelays(by searchString: String) {
        currentSearchString = searchString

        var list: [[LocationCellViewModel]] = []
        for section in 0 ..< SelectLocationSection.allCases.count {
            list.append(
                dataSources[section]
                    .search(by: searchString)
                    .map { LocationCellViewModel(group: SelectLocationSection.allCases[section], location: $0) }
            )
        }

        updateDataSnapshot(with: list, reloadExisting: !searchString.isEmpty)

        if searchString.isEmpty {
            setSelectedRelayLocation(selectedRelayLocation, animated: false, completion: {
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
        var snapshot = NSDiffableDataSourceSnapshot<SelectLocationSection, LocationCellViewModel>()

        let sections = SelectLocationSection.allCases
        snapshot.appendSections(sections)
        for (index, section) in sections.enumerated() {
            snapshot.appendItems(list[index], toSection: section)
        }

        if reloadExisting {
            snapshot.reloadSections(SelectLocationSection.allCases)
        }

        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func registerClasses() {
        SelectLocationSection.allCases.forEach {
            tableView.register(
                $0.cell.reusableViewClass,
                forCellReuseIdentifier: $0.cell.reuseIdentifier
            )
        }
    }

    private func setSelectedRelayLocation(
        _ relayLocation: LocationCellViewModel?,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        selectedRelayLocation = relayLocation
        selectedRelayLocation
            .flatMap { item in
                let group = item.group
                var locationList = snapshot().itemIdentifiers(inSection: group)
                guard !locationList.contains(item) else {
                    completion?()
                    return
                }
                let selectedLocationTree = item.location.ascendants + [item.location]

                guard let first = selectedLocationTree.first else { return }
                let topLocation = LocationCellViewModel(group: group, location: first)

                guard let indexPath = indexPath(for: topLocation),
                      let topNode = node(for: topLocation) else {
                    return
                }

                selectedLocationTree.forEach { location in
                    node(for: LocationCellViewModel(group: group, location: location))?.showsChildren = true
                }

                locationList.addLocations(
                    topNode.flatRelayLocationList().map { LocationCellViewModel(group: group, location: $0) },
                    at: indexPath.row + 1
                )

                var list: [[LocationCellViewModel]] = Array(repeating: [], count: dataSources.count)
                for index in 0 ..< list.count {
                    list[index] = (index == indexPath.section)
                        ? locationList
                        : snapshot().itemIdentifiers(inSection: SelectLocationSection.allCases[index])
                }

                updateDataSnapshot(
                    with: list,
                    reloadExisting: true,
                    animated: animated,
                    completion: completion
                )
            }
    }
}

extension LocationDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        itemIdentifier(for: indexPath).flatMap { node(for: $0) }?.isActive ?? false
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        itemIdentifier(for: indexPath).flatMap { node(for: $0) }?.indentationLevel ?? 0
    }

    func tableView(
        _ tableView: UITableView,
        willDisplay cell: UITableViewCell,
        forRowAt indexPath: IndexPath
    ) {
        if let item = itemIdentifier(for: indexPath),
           item == selectedRelayLocation {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        itemIdentifier(for: indexPath)
            .flatMap { item in
                guard item.location != selectedRelayLocation?.location else { return }
                didSelectRelayLocation?(item.location)

                setSelectedRelayLocation(item, animated: false)

                indexPathForSelectedRelay().flatMap {
                    let cell = tableView.cellForRow(at: $0)
                    cell?.setSelected(false, animated: false)
                }
            }
    }
}

extension LocationDataSource: LocationCellEventHandler {
    func toggleCell(for item: LocationCellViewModel) {
        indexPath(for: item).flatMap { indexPath in
            guard let node = node(for: item), let cell = tableView.cellForRow(at: indexPath) else { return }

            let isExpanded = node.showsChildren
            let group = SelectLocationSection.allCases[indexPath.section]

            node.showsChildren = !isExpanded
            locationCellFactory.configureCell(
                cell,
                item: LocationCellViewModel(group: group, location: node.location),
                indexPath: indexPath
            )

            var locationList = snapshot().itemIdentifiers(inSection: group)
            let locationsToEdit = node.flatRelayLocationList().map { LocationCellViewModel(group: group, location: $0) }

            if !isExpanded {
                locationList.addLocations(locationsToEdit, at: indexPath.row + 1)
            } else {
                locationsToEdit.forEach { self.node(for: $0)?.showsChildren = false }
                locationList.removeLocations(locationsToEdit)
            }

            var list: [[LocationCellViewModel]] = Array(repeating: [], count: dataSources.count)
            for index in 0 ..< list.count {
                list[index] = (index == indexPath.section)
                    ? locationList
                    : snapshot().itemIdentifiers(inSection: SelectLocationSection.allCases[index])
            }

            updateDataSnapshot(with: list, completion: {
                self.scroll(to: item, animated: true)
            })
        }
    }

    func node(for item: LocationCellViewModel) -> SelectLocationNode? {
        guard let sectionIndex = SelectLocationSection.allCases.firstIndex(of: item.group) else {
            return nil
        }
        return dataSources[sectionIndex].nodeByLocation[item.location]
    }
}

extension LocationDataSource {
    private func scroll(to location: LocationCellViewModel, animated: Bool) {
        guard let visibleIndexPaths = tableView.indexPathsForVisibleRows,
              let indexPath = indexPath(for: location),
              let node = node(for: location) else { return }

        if node.children.count > visibleIndexPaths.count {
            tableView.scrollToRow(at: indexPath, at: .top, animated: animated)
        } else {
            node.children.last.flatMap { last in
                if let lastInsertedIndexPath = self.indexPath(for: LocationCellViewModel(
                    group: SelectLocationSection.allCases[indexPath.section],
                    location: last.location
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
        indexPathForSelectedRelay().flatMap {
            tableView.scrollToRow(at: $0, at: .middle, animated: false)
        }
    }
}

private extension [LocationCellViewModel] {
    mutating func addLocations(_ locations: [LocationCellViewModel], at index: Int) {
        if index < count {
            insert(contentsOf: locations, at: index)
        } else {
            append(contentsOf: locations)
        }
    }

    mutating func removeLocations(_ locations: [LocationCellViewModel]) {
        removeAll(where: { location in
            locations.contains(location)
        })
    }
}
