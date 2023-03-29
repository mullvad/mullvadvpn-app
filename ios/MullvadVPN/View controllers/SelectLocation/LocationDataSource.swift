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
    private var filteredLocationList = [RelayLocation]()
    private var rootNode = makeRootNode()
    private(set) var selectedRelayLocation: RelayLocation?

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

    func setSelectedRelayLocation(
        _ relayLocation: RelayLocation?,
        animated: Bool
    ) {
        selectedRelayLocation = relayLocation

        let selectedLocationTree = selectedRelayLocation?.ascendants ?? []
        selectedLocationTree.forEach { location in
            nodeByLocation[location]?.showsChildren = true
        }

        updateCellFactory(with: nodeByLocation)
        updateDataSnapshot(with: locationList, animated: animated)
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
        self.rootNode = rootNode
        locationList = rootNode.flatRelayLocationList()
        filteredLocationList = locationList

        updateCellFactory(with: nodeByLocation)
        updateDataSnapshot(with: locationList)
    }

    func indexPathForSelectedRelay() -> IndexPath? {
        return selectedRelayLocation.flatMap { indexPath(for: $0) }
    }

    func filterRelays(by searchString: String) {
        tableView.scrollRectToVisible(.zero, animated: false)

        guard searchString.count > 1 else {
            filteredLocationList = locationList
            updateDataSnapshot(with: filteredLocationList)
            return
        }

        var locations = [RelayLocation]()

        locationList.forEach { location in
            guard let countryNode = nodeByLocation[location] else { return }

            if countryNode.displayName.fuzzyMatch(searchString) {
                locations.append(countryNode.location)
            }

            for cityNode in countryNode.children {
                if cityNode.displayName.fuzzyMatch(searchString) {
                    if !locations.contains(countryNode.location) {
                        locations.append(countryNode.location)
                    }

                    locations.append(cityNode.location)

                    for relayNode in cityNode.children {
                        if relayNode.displayName.fuzzyMatch(searchString) {
                            if !locations.contains(cityNode.location) {
                                locations.append(cityNode.location)
                            }

                            locations.append(relayNode.location)
                        }
                    }
                }
            }
        }

        filteredLocationList = locations
        updateDataSnapshot(with: filteredLocationList)
    }

    private func updateDataSnapshot(
        with locations: [RelayLocation],
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<Int, RelayLocation>()
        snapshot.appendSections([0])

        for location in locations {
            snapshot.appendItems([location])

            guard let countryNode = nodeByLocation[location], countryNode.showsChildren else {
                continue
            }

            for cityNode in countryNode.children {
                snapshot.appendItems([cityNode.location])

                guard cityNode.showsChildren else {
                    continue
                }

                snapshot.appendItems(cityNode.children.map { $0.location })
            }
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

    private func toggleChildren(
        _ relayLocation: RelayLocation,
        show: Bool,
        animated: Bool
    ) {
        guard let node = nodeByLocation[relayLocation] else { return }

        node.showsChildren = show

        if let indexPath = indexPath(for: node.location),
           let cell = tableView.cellForRow(at: indexPath)
        {
            locationCellFactory.configureCell(cell, item: node.location, indexPath: indexPath)
        }

        updateCellFactory(with: nodeByLocation)
        updateDataSnapshot(with: filteredLocationList, animated: animated) { [weak self] in
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

    private func item(for indexPath: IndexPath) -> LocationDataSourceItemProtocol? {
        return itemIdentifier(for: indexPath).flatMap { nodeByLocation[$0] }
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
        guard let item = item(for: indexPath) else { return }

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

        func flatRelayLocationList() -> [RelayLocation] {
            return children.reduce(into: []) { array, node in
                Self.flatten(node: node, into: &array)
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

        private class func flatten(node: Node, into array: inout [RelayLocation]) {
            array.append(node.location)
            if node.showsChildren {
                for child in node.children {
                    Self.flatten(node: child, into: &array)
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

private extension String {
    func fuzzyMatch(_ needle: String) -> Bool {
        guard !needle.isEmpty else {
            return true
        }

        let haystack = lowercased()
        let needle = needle.lowercased()

        var indices: [Index] = []
        var remainder = needle[...].utf8

        for index in haystack.utf8.indices {
            let character = haystack.utf8[index]
            if character == remainder[remainder.startIndex] {
                indices.append(index)
                remainder.removeFirst()

                if remainder.isEmpty {
                    return !indices.isEmpty
                }
            }
        }

        return false
    }
}
