//
//  LocationDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import UIKit

protocol LocationDataSourceItemProtocol {
    var location: RelayLocation { get }
    var displayName: String { get }
    var showsChildren: Bool { get }
    var isActive: Bool { get }

    var isCollapsible: Bool { get }
    var indentationLevel: Int { get }
}

final class LocationDataSource: UITableViewDiffableDataSource<Int, RelayLocation> {
    enum CellReuseIdentifiers: String, CaseIterable {
        case locationCell

        var reusableViewClass: AnyClass {
            switch self {
            case .locationCell:
                return SelectLocationCell.self
            }
        }
    }

    private var nodeByLocation = [RelayLocation: Node]()
    private var locationList = [RelayLocation]()
    private var currentSearchString = ""

    private let tableView: UITableView
    private let locationCellFactory: LocationCellFactory

    private class func makeRootNode() -> Node {
        return Node(
            type: .root,
            location: RelayLocation.country("#root"),
            displayName: "",
            showsChildren: true,
            isActive: true,
            children: []
        )
    }

    var selectedRelayLocation: RelayLocation?
    var didSelectRelayLocation: ((RelayLocation) -> Void)?

    init(tableView: UITableView) {
        self.tableView = tableView

        let locationCellFactory = LocationCellFactory(
            tableView: tableView,
            nodeByLocation: nodeByLocation
        )
        self.locationCellFactory = locationCellFactory

        super.init(tableView: tableView) { tableView, indexPath, itemIdentifier in
            locationCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.delegate = self
        locationCellFactory.delegate = self

        defaultRowAnimation = .fade
        registerClasses()
    }

    func setRelays(_ response: REST.ServerRelaysResponse) {
        let rootNode = Self.makeRootNode()
        var nodeByLocation = [RelayLocation: Node]()

        for relay in response.wireguard.relays {
            guard case let .city(
                countryCode,
                cityCode
            ) = RelayLocation(dashSeparatedString: relay.location),
                let serverLocation = response.locations[relay.location] else { continue }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ascendantOrSelf in relayLocation.ascendants + [relayLocation] {
                guard !nodeByLocation.keys.contains(ascendantOrSelf) else {
                    continue
                }

                // Maintain the `showsChildren` state when transitioning between relay lists
                let wasShowingChildren = nodeByLocation[ascendantOrSelf]?
                    .showsChildren ?? false

                let node: Node
                switch ascendantOrSelf {
                case .country:
                    node = Node(
                        type: .country,
                        location: ascendantOrSelf,
                        displayName: serverLocation.country,
                        showsChildren: wasShowingChildren,
                        isActive: true,
                        children: []
                    )
                    rootNode.addChild(node)

                case let .city(countryCode, _):
                    node = Node(
                        type: .city,
                        location: ascendantOrSelf,
                        displayName: serverLocation.city,
                        showsChildren: wasShowingChildren,
                        isActive: true,
                        children: []
                    )
                    nodeByLocation[.country(countryCode)]!.addChild(node)

                case let .hostname(countryCode, cityCode, _):
                    node = Node(
                        type: .relay,
                        location: ascendantOrSelf,
                        displayName: relay.hostname,
                        showsChildren: false,
                        isActive: relay.active,
                        children: []
                    )
                    nodeByLocation[.city(countryCode, cityCode)]!.addChild(node)
                }

                nodeByLocation[ascendantOrSelf] = node
            }
        }

        rootNode.sortChildrenRecursive()
        rootNode.computeActiveChildrenRecursive()
        self.nodeByLocation = nodeByLocation
        locationList = rootNode.flatRelayLocationList()

        filterRelays(by: currentSearchString)
    }

    func indexPathForSelectedRelay() -> IndexPath? {
        return selectedRelayLocation.flatMap { indexPath(for: $0) }
    }

    func filterRelays(by searchString: String) {
        currentSearchString = searchString

        if currentSearchString.isEmpty {
            return resetLocationList()
        }

        var filteredLocations = [RelayLocation]()

        locationList.forEach { location in
            guard let countryNode = nodeByLocation[location] else { return }
            countryNode.showsChildren = false

            if searchString.isEmpty || countryNode.displayName.fuzzyMatch(searchString) {
                filteredLocations.append(countryNode.location)
            }

            for cityNode in countryNode.children {
                cityNode.showsChildren = false

                if cityNode.displayName.fuzzyMatch(searchString) {
                    if !filteredLocations.contains(countryNode.location) {
                        filteredLocations.append(countryNode.location)
                    }

                    filteredLocations.append(cityNode.location)
                    countryNode.showsChildren = true

                    if cityNode.children.contains(where: { node in
                        node.displayName.fuzzyMatch(searchString)
                    }) {
                        filteredLocations.append(contentsOf: cityNode.children.map { $0.location })
                        cityNode.showsChildren = true
                    }
                }
            }
        }

        updateDataSnapshot(with: filteredLocations, reloadExisting: true) { [weak self] in
            self?.scrollToTop(animated: false)
        }
    }

    private func updateDataSnapshot(
        with locations: [RelayLocation],
        reloadExisting: Bool = false,
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        updateCellFactory(with: nodeByLocation)

        var snapshot = NSDiffableDataSourceSnapshot<Int, RelayLocation>()

        snapshot.appendSections([0])
        snapshot.appendItems(locations)

        if reloadExisting {
            snapshot.reloadItems(locations)
        }

        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { enumCase in
            tableView.register(
                enumCase.reusableViewClass,
                forCellReuseIdentifier: enumCase.rawValue
            )
        }
    }

    private func updateCellFactory(with nodeByLocation: [RelayLocation: Node]) {
        locationCellFactory.nodeByLocation = nodeByLocation
    }

    private func setSelectedRelayLocation(
        _ relayLocation: RelayLocation?,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        selectedRelayLocation = relayLocation
        var locationList = snapshot().itemIdentifiers

        guard let selectedRelayLocation = selectedRelayLocation,
              !locationList.contains(selectedRelayLocation) else { return }

        let selectedLocationTree = selectedRelayLocation.ascendants + [selectedRelayLocation]

        guard let topLocation = selectedLocationTree.first,
              let topNode = nodeByLocation[topLocation],
              let indexPath = indexPath(for: topLocation)
        else {
            return
        }

        selectedLocationTree.forEach { location in
            nodeByLocation[location]?.showsChildren = true
        }

        locationList.addLocations(topNode.flatRelayLocationList(), at: indexPath.row + 1)
        updateDataSnapshot(with: locationList, reloadExisting: true, animated: animated, completion: completion)
    }

    private func toggleChildren(
        _ relayLocation: RelayLocation,
        show: Bool,
        animated: Bool
    ) {
        guard let node = nodeByLocation[relayLocation],
              let indexPath = indexPath(for: node.location),
              let cell = tableView.cellForRow(at: indexPath) else { return }

        node.showsChildren = show
        locationCellFactory.configureCell(cell, item: node.location, indexPath: indexPath)

        var locationList = snapshot().itemIdentifiers
        let locationsToEdit = node.flatRelayLocationList()

        if show {
            locationList.addLocations(locationsToEdit, at: indexPath.row + 1)
        } else {
            locationsToEdit.forEach { nodeByLocation[$0]?.showsChildren = false }
            locationList.removeLocations(locationsToEdit)
        }

        updateDataSnapshot(with: locationList, animated: animated) { [weak self] in
            guard let visibleIndexPaths = self?.tableView.indexPathsForVisibleRows else { return }

            let scrollToNodeTop = {
                if let firstInsertedIndexPath = self?.indexPath(for: node.location) {
                    self?.tableView.scrollToRow(
                        at: firstInsertedIndexPath,
                        at: .top,
                        animated: animated
                    )
                }
            }

            let scrollToNodeBottom = {
                if let location = node.children.last?.location,
                   let lastInsertedIndexPath = self?.indexPath(for: location),
                   let lastVisibleIndexPath = visibleIndexPaths.last,
                   lastInsertedIndexPath >= lastVisibleIndexPath
                {
                    self?.tableView.scrollToRow(
                        at: lastInsertedIndexPath,
                        at: .bottom,
                        animated: animated
                    )
                }
            }

            if node.children.count > visibleIndexPaths.count {
                scrollToNodeTop()
            } else {
                scrollToNodeBottom()
            }
        }
    }

    private func resetLocationList() {
        nodeByLocation.values.forEach { $0.showsChildren = false }

        updateDataSnapshot(with: locationList, reloadExisting: true)
        setSelectedRelayLocation(selectedRelayLocation, animated: false)

        if let indexPath = indexPathForSelectedRelay() {
            tableView.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    private func item(for indexPath: IndexPath) -> LocationDataSourceItemProtocol? {
        return itemIdentifier(for: indexPath).flatMap { nodeByLocation[$0] }
    }

    private func scrollToTop(animated: Bool) {
        tableView.setContentOffset(.zero, animated: animated)
    }
}

extension LocationDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return item(for: indexPath)?.isActive ?? false
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        return item(for: indexPath)?.indentationLevel ?? 0
    }

    func tableView(
        _ tableView: UITableView,
        willDisplay cell: UITableViewCell,
        forRowAt indexPath: IndexPath
    ) {
        if let item = item(for: indexPath),
           item.location == selectedRelayLocation
        {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = item(for: indexPath),
              item.location != selectedRelayLocation
        else {
            return
        }

        if let indexPath = indexPathForSelectedRelay(),
           let cell = tableView.cellForRow(at: indexPath)
        {
            cell.setSelected(false, animated: false)
        }

        setSelectedRelayLocation(
            item.location,
            animated: false
        )

        didSelectRelayLocation?(item.location)
    }
}

extension LocationDataSource: LocationCellEventHandler {
    func collapseCell(for item: RelayLocation) {
        guard let node = nodeByLocation[item] else { return }

        toggleChildren(
            item,
            show: !node.showsChildren,
            animated: true
        )
    }
}

extension LocationDataSource {
    enum NodeType {
        case root
        case country
        case city
        case relay
    }

    class Node: LocationDataSourceItemProtocol {
        let nodeType: NodeType
        var location: RelayLocation
        var displayName: String
        var showsChildren: Bool
        var isActive: Bool
        var children: [Node]

        var isCollapsible: Bool {
            switch nodeType {
            case .country, .city:
                return true
            case .root, .relay:
                return false
            }
        }

        var indentationLevel: Int {
            switch nodeType {
            case .root, .country:
                return 0
            case .city:
                return 1
            case .relay:
                return 2
            }
        }

        init(
            type: NodeType,
            location: RelayLocation,
            displayName: String,
            showsChildren: Bool,
            isActive: Bool,
            children: [Node]
        ) {
            nodeType = type
            self.location = location
            self.displayName = displayName
            self.showsChildren = showsChildren
            self.isActive = isActive
            self.children = children
        }

        func addChild(_ child: Node) {
            children.append(child)
        }

        func sortChildrenRecursive() {
            sortChildren()
            children.forEach { node in
                node.sortChildrenRecursive()
            }
        }

        func computeActiveChildrenRecursive() {
            switch nodeType {
            case .root, .country:
                for node in children {
                    node.computeActiveChildrenRecursive()
                }
                fallthrough
            case .city:
                isActive = children.contains(where: { node -> Bool in
                    return node.isActive
                })
            case .relay:
                break
            }
        }

        func countChildrenRecursive(where condition: @escaping (Node) -> Bool) -> Int {
            return children.reduce(into: 0) { numVisibleChildren, node in
                numVisibleChildren += 1
                if condition(node) {
                    numVisibleChildren += node.countChildrenRecursive(where: condition)
                }
            }
        }

        func flatRelayLocationList(includeHiddenChildren: Bool = false) -> [RelayLocation] {
            return children.reduce(into: []) { array, node in
                Self.flatten(node: node, into: &array, includeHiddenChildren: includeHiddenChildren)
            }
        }

        private func sortChildren() {
            switch nodeType {
            case .root, .country:
                children.sort { a, b -> Bool in
                    return lexicalSortComparator(a.displayName, b.displayName)
                }
            case .city:
                children.sort { a, b -> Bool in
                    return fileSortComparator(
                        a.location.stringRepresentation,
                        b.location.stringRepresentation
                    )
                }
            case .relay:
                break
            }
        }

        private class func flatten(node: Node, into array: inout [RelayLocation], includeHiddenChildren: Bool) {
            array.append(node.location)
            if includeHiddenChildren || node.showsChildren {
                for child in node.children {
                    Self.flatten(node: child, into: &array, includeHiddenChildren: includeHiddenChildren)
                }
            }
        }
    }
}

private func lexicalSortComparator(_ a: String, _ b: String) -> Bool {
    return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
}

private func fileSortComparator(_ a: String, _ b: String) -> Bool {
    return a.localizedStandardCompare(b) == .orderedAscending
}

private extension Array where Element == RelayLocation {
    mutating func addLocations(_ locations: [RelayLocation], at index: Int) {
        if index < count {
            insert(contentsOf: locations, at: index)
        } else {
            append(contentsOf: locations)
        }
    }

    mutating func removeLocations(_ locations: [RelayLocation]) {
        removeAll(where: { location in
            locations.contains(location)
        })
    }
}
