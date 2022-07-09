//
//  LocationDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol LocationDataSourceItemProtocol {
    var location: RelayLocation { get }
    var displayName: String { get }
    var showsChildren: Bool { get }
    var isActive: Bool { get }
    var isPinned: Bool { get }

    var isCollapsible: Bool { get }
    var isPinnable: Bool { get }
    var indentationLevel: Int { get }
}

class LocationDataSource: NSObject, UITableViewDataSource {
    private var nodeByLocation = [RelayLocation: Node]()
    private var locationList = [RelayLocation]()
    private var rootNode = makeRootNode()

    /// The root node that holds filtered relay locations during the search.
    private var searchResultNode: Node?

    typealias CellProviderBlock = (UITableView, IndexPath, LocationDataSourceItemProtocol)
        -> UITableViewCell?

    private let tableView: UITableView
    private let cellProvider: CellProviderBlock

    private(set) var selectedRelayLocation: RelayLocation?
    private var lastShowHiddenParents = false
    private var lastScrollPosition: UITableView.ScrollPosition = .none

    private class func makeRootNode() -> Node {
        return Node(
            type: .root,
            location: RelayLocation.country("#root"),
            displayName: "",
            showsChildren: true,
            isActive: true,
            isPinned: false,
            children: []
        )
    }

    init(tableView: UITableView, cellProvider: @escaping CellProviderBlock) {
        self.tableView = tableView
        self.cellProvider = cellProvider
        super.init()

        tableView.dataSource = self
    }

    func setSelectedRelayLocation(
        _ relayLocation: RelayLocation?,
        showHiddenParents: Bool,
        animated: Bool,
        scrollPosition: UITableView.ScrollPosition,
        completion: (() -> Void)? = nil
    ) {
        selectedRelayLocation = relayLocation
        lastShowHiddenParents = showHiddenParents
        lastScrollPosition = scrollPosition

        if relayLocation == nil {
            if let indexPath = tableView.indexPathForSelectedRow {
                tableView.deselectRow(at: indexPath, animated: animated)
            }
            completion?()
        } else {
            let setSelection = {
                if let indexPath = self.indexPathForSelectedRelay() {
                    self.tableView.selectRow(
                        at: indexPath,
                        animated: animated,
                        scrollPosition: scrollPosition
                    )
                }
                completion?()
            }

            if let relayLocation = relayLocation, showHiddenParents {
                showParents(relayLocation, animated: animated, completion: setSelection)
            } else {
                setSelection()
            }
        }
    }

    func setRelays(_ response: REST.ServerRelaysResponse) {
        let rootNode = Self.makeRootNode()
        var nodeByLocation = [RelayLocation: Node]()
        let dataSourceWasEmpty = locationList.isEmpty

        let pinnedLocationNames: Set<String>
        do {
            pinnedLocationNames = try SettingsManager.getPinnedLocationNames()
        } catch {
            pinnedLocationNames = []
            // Unhandled read pinned locations error
            print(error)
        }

        for relay in response.wireguard.relays {
            guard case let .city(
                countryCode,
                cityCode
            ) = RelayLocation(dashSeparatedString: relay.location),
                let serverLocation = response.locations[relay.location] else {
                continue
            }

            let relayLocation = RelayLocation.hostname(countryCode, cityCode, relay.hostname)

            for ascendantOrSelf in relayLocation.ascendants + [relayLocation] {
                guard !nodeByLocation.keys.contains(ascendantOrSelf) else {
                    continue
                }

                // Maintain the `showsChildren` state when transitioning between relay lists
                let wasShowingChildren = self.nodeByLocation[ascendantOrSelf]?
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
                        isPinned: pinnedLocationNames.contains(serverLocation.country),
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
                        isPinned: pinnedLocationNames.contains(serverLocation.city),
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
                        isPinned: pinnedLocationNames.contains(relay.hostname),
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

        tableView.reloadData()

        let setSelection = { (_ scrollPosition: UITableView.ScrollPosition) in
            if let indexPath = self.indexPathForSelectedRelay() {
                self.tableView.selectRow(
                    at: indexPath,
                    animated: false,
                    scrollPosition: scrollPosition
                )
            }
        }

        // Sometimes the selected relay may be set before the data source is populated with relays.
        // In that case restore the selection using cached parameters from the last call to
        // `setSelectedRelayLocation`.
        if let selectedRelayLocation = selectedRelayLocation, dataSourceWasEmpty {
            if lastShowHiddenParents {
                showParents(selectedRelayLocation, animated: false) {
                    setSelection(self.lastScrollPosition)
                }
            } else {
                setSelection(lastScrollPosition)
            }
        } else {
            setSelection(.none)
        }
    }

    func showChildren(
        _ relayLocation: RelayLocation,
        showHiddenParents: Bool = false,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        toggleChildrenInternal(
            relayLocation,
            show: true,
            showHiddenParents: showHiddenParents,
            animated: animated,
            completion: completion
        )
    }

    func hideChildren(
        _ relayLocation: RelayLocation,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        toggleChildrenInternal(
            relayLocation,
            show: false,
            showHiddenParents: false,
            animated: animated,
            completion: completion
        )
    }

    func toggleChildren(
        _ relayLocation: RelayLocation,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        guard let node = nodeByLocation[relayLocation] else {
            return
        }

        toggleChildrenInternal(
            relayLocation,
            show: !node.showsChildren,
            showHiddenParents: false,
            animated: animated,
            completion: completion
        )
    }

    func filterLocations(by searchText: String) {

        // Reset root node and location list on empty search text
        guard !searchText.isEmpty else {
            rootNode.sortChildrenRecursive()
            locationList = rootNode.flatRelayLocationList()
            searchResultNode = nil
            tableView.reloadData()
            return
        }

        // Filter nodes by search text
        let searchResultChildren = nodeByLocation
            .compactMap { element -> Node? in
                let node = element.value
                guard node.nodeType != .relay else {
                    return nil
                }
                guard node.displayName.hasPrefixCaseDiacriticInsensitive(searchText) else {
                    return nil
                }
                return node
            }

        // Configure search root node and location list
        let searchResultNode = Self.makeRootNode()
        searchResultNode.children = searchResultChildren
        searchResultNode.sortChildrenRecursive()
        self.searchResultNode = searchResultNode
        locationList = searchResultNode.flatRelayLocationList()

        // Update table view
        tableView.reloadData()
        if !searchResultChildren.isEmpty {
            tableView.scrollToRow(at: .init(row: 0, section: 0), at: .middle, animated: false)
        }
    }

    func togglePin(_ relayLocation: RelayLocation) {

        guard let node = nodeByLocation[relayLocation] else {
            return
        }

        node.isPinned.toggle()

        // Use `searchResultNode` to sync indices before and after pinning during the search
        let currentRootNode = searchResultNode ?? rootNode
        currentRootNode.sortChildrenRecursive()
        let newLocationList = currentRootNode.flatRelayLocationList()

        // Calculate table view changes
        let relayLocationsToMove = [relayLocation] + node.flatRelayLocationList()
        let changes = relayLocationsToMove
            .compactMap { relayLocation -> (IndexPath, IndexPath)? in
                let fromIndexPath = locationList
                    .firstIndex(of: relayLocation)
                    .flatMap { row -> IndexPath in
                        IndexPath(row: row, section: 0)
                    }
                let toIndexPath = newLocationList
                    .firstIndex(of: relayLocation)
                    .flatMap { row -> IndexPath in
                        IndexPath(row: row, section: 0)
                    }
                guard
                    let fromIndexPath = fromIndexPath,
                    let toIndexPath = toIndexPath
                else {
                    return nil
                }
                return (fromIndexPath, toIndexPath)
            }

        // Update re-sorted list and table view
        locationList = newLocationList
        tableView.performBatchUpdates {
            changes.forEach(tableView.moveRow)
        } completion: { [weak self] finished in
            self?.tableView.reloadData()
        }

        // Save pinned locations
        do {
            let pinnedLocationNames = nodeByLocation
                .reduce(into: Set<String>()) { result, element in
                    if element.value.isPinned {
                        result.insert(element.value.displayName)
                    }
                }
            try SettingsManager.setPinnedLocationNames(pinnedLocationNames)
        } catch {
            // Unhandled write pinned locations error
            print(error)
        }
    }

    private func showParents(
        _ relayLocation: RelayLocation,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        switch relayLocation {
        case .country:
            completion?()
        case .city:
            if let countryLocation = relayLocation.ascendants.first {
                toggleChildrenInternal(
                    countryLocation,
                    show: true,
                    showHiddenParents: false,
                    animated: animated,
                    completion: completion
                )
            }
        case .hostname:
            if let cityLocation = relayLocation.ascendants.last {
                toggleChildrenInternal(
                    cityLocation,
                    show: true,
                    showHiddenParents: true,
                    animated: animated,
                    completion: completion
                )
            }
        }
    }

    private func toggleChildrenInternal(
        _ relayLocation: RelayLocation,
        show: Bool,
        showHiddenParents: Bool,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        let affectedRelayLocations: [RelayLocation]
        if showHiddenParents {
            affectedRelayLocations = relayLocation.ascendants + [relayLocation]
        } else {
            affectedRelayLocations = [relayLocation]
        }

        let affectedNodes = affectedRelayLocations.compactMap { relayLocation -> Node? in
            return nodeByLocation[relayLocation]
        }

        // Pick the topmost node to expand or collapse
        guard let topNode = affectedNodes.first(where: { node -> Bool in
            return node.isCollapsible && node.showsChildren != show
        }) else {
            completion?()
            return
        }

        let numAffectedChildren = topNode.countChildrenRecursive { node -> Bool in
            if show {
                return node.showsChildren || affectedNodes.contains(where: { otherNode -> Bool in
                    return node === otherNode
                })
            } else {
                return node.showsChildren
            }
        }

        let applyChanges = { () -> ChangeSet? in
            guard let topIndexPath = self.indexPath(for: topNode.location) else {
                return nil
            }

            affectedNodes.forEach { node in
                node.showsChildren = show
            }

            let affectedRange = (topIndexPath.row + 1 ... topIndexPath.row + numAffectedChildren)
            let affectedIndexPaths = affectedRange.map { row -> IndexPath in
                return IndexPath(row: row, section: 0)
            }

            if show {
                self.locationList.insert(
                    contentsOf: topNode.flatRelayLocationList(),
                    at: topIndexPath.row + 1
                )

                return ChangeSet(
                    insertIndexPaths: affectedIndexPaths,
                    deleteIndexPaths: [],
                    updateIndexPaths: [topIndexPath]
                )
            } else {
                self.locationList.removeSubrange(affectedRange)

                return ChangeSet(
                    insertIndexPaths: [],
                    deleteIndexPaths: affectedIndexPaths,
                    updateIndexPaths: [topIndexPath]
                )
            }
        }

        let restoreSelection = {
            if let indexPath = self.indexPathForSelectedRelay() {
                self.tableView.selectRow(at: indexPath, animated: false, scrollPosition: .none)
            }
        }

        if animated {
            tableView.performBatchUpdates {
                if let changeSet = applyChanges() {
                    tableView.insertRows(at: changeSet.insertIndexPaths, with: .fade)
                    tableView.deleteRows(at: changeSet.deleteIndexPaths, with: .fade)
                    tableView.reloadRows(at: changeSet.updateIndexPaths, with: .none)
                }
            } completion: { finished in
                restoreSelection()
                completion?()
            }
        } else {
            _ = applyChanges()
            tableView.reloadData()
            restoreSelection()
            completion?()
        }
    }

    func relayLocation(for indexPath: IndexPath) -> RelayLocation? {
        return locationList[indexPath.row]
    }

    func item(for indexPath: IndexPath) -> LocationDataSourceItemProtocol? {
        return relayLocation(for: indexPath)
            .flatMap { relayLocation -> Node? in
                return nodeByLocation[relayLocation]
            }
    }

    func indexPath(for location: RelayLocation) -> IndexPath? {
        return locationList.firstIndex(of: location).map { index -> IndexPath in
            return IndexPath(row: index, section: 0)
        }
    }

    func indexPathForSelectedRelay() -> IndexPath? {
        return selectedRelayLocation.flatMap { relayLocation -> IndexPath? in
            return self.indexPath(for: relayLocation)
        }
    }

    // MARK: - UITableViewDataSource

    func numberOfSections(in tableView: UITableView) -> Int {
        return 1
    }

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        assert(section == 0)
        return locationList.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        assert(indexPath.section == 0)
        let item = item(for: indexPath)!
        return cellProvider(tableView, indexPath, item)!
    }
}

extension LocationDataSource {
    private enum NodeType {
        case root
        case country
        case city
        case relay
    }

    private class Node: LocationDataSourceItemProtocol {
        let nodeType: NodeType
        var location: RelayLocation
        var displayName: String
        var showsChildren: Bool
        var isActive: Bool
        var isPinned: Bool
        var children: [Node]

        var isCollapsible: Bool {
            switch nodeType {
            case .country, .city:
                return true
            case .root, .relay:
                return false
            }
        }

        var isPinnable: Bool {
            switch nodeType {
            case .country:
                return true
            case .root, .city, .relay:
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
            isPinned: Bool,
            children: [Node]
        ) {
            nodeType = type
            self.location = location
            self.displayName = displayName
            self.showsChildren = showsChildren
            self.isActive = isActive
            self.isPinned = isPinned
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
                    if a.isPinned && b.isPinned {
                        return lexicalSortComparator(a.displayName, b.displayName)
                    }
                    if a.isPinned {
                        return true
                    }
                    if b.isPinned {
                        return false
                    }
                    return lexicalSortComparator(a.displayName, b.displayName)
                }
            case .city:
                children.sort { a, b -> Bool in
                    if a.isPinned && b.isPinned {
                        return fileSortComparator(
                            a.location.stringRepresentation,
                            b.location.stringRepresentation
                        )
                    }
                    if a.isPinned {
                        return true
                    }
                    if b.isPinned {
                        return false
                    }
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

    private struct ChangeSet {
        let insertIndexPaths: [IndexPath]
        let deleteIndexPaths: [IndexPath]
        let updateIndexPaths: [IndexPath]
    }
}

private func lexicalSortComparator(_ a: String, _ b: String) -> Bool {
    return a.localizedCaseInsensitiveCompare(b) == .orderedAscending
}

private func fileSortComparator(_ a: String, _ b: String) -> Bool {
    return a.localizedStandardCompare(b) == .orderedAscending
}

private extension String {

    func hasPrefixCaseDiacriticInsensitive<T>(_ other: T) -> Bool where T: StringProtocol {
        let options: String.CompareOptions = [.caseInsensitive, .diacriticInsensitive]
        return range(of: other, options: options)?.lowerBound == startIndex
    }
}
