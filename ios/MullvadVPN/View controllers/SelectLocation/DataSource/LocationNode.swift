//
//  LocationNode.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

@Observable
class LocationNode: @unchecked Sendable {
    let name: String
    var code: String
    var locations: [RelayLocation]
    var isActive: Bool
    weak var parent: LocationNode?
    var children: [LocationNode]
    var showsChildren: Bool
    var isHiddenFromSearch: Bool
    var isConnected: Bool
    var isSelected: Bool
    var isExcluded: Bool
    let id = UUID()

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        isActive: Bool = true,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false,
        isConnected: Bool = false,
        isSelected: Bool = false,
        isExcluded: Bool = false
    ) {
        self.name = name
        self.code = code
        self.locations = locations
        self.isActive = isActive
        self.parent = parent
        self.children = children
        self.showsChildren = showsChildren
        self.isHiddenFromSearch = isHiddenFromSearch
        self.isConnected = isConnected
        self.isSelected = isSelected
        self.isExcluded = isExcluded
    }

    /// Recursively copies a node, its parent and its descendants from another
    /// node (tree), with an optional custom root parent.
    func copy(withParent parent: LocationNode? = nil) -> LocationNode {
        let node = LocationNode(
            name: name,
            code: code,
            locations: locations,
            isActive: isActive,
            parent: parent,
            children: [],
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch,
            isConnected: isConnected,
            isSelected: false,  // explicity set to false since it's a different node
            isExcluded: isExcluded
        )

        node.children = recursivelyCopyChildren(withParent: node)

        return node
    }
}

extension LocationNode {
    var root: LocationNode {
        parent?.root ?? self
    }

    var asRecentLocationNode: RecentLocationNode? {
        self as? RecentLocationNode
    }
    var asCustomListNode: CustomListLocationNode? {
        self as? CustomListLocationNode
    }

    var userSelectedRelays: UserSelectedRelays {
        var customListSelection: UserSelectedRelays.CustomListSelection?
        if let topmostNode = root.asCustomListNode {
            customListSelection = UserSelectedRelays.CustomListSelection(
                listId: topmostNode.customList.id,
                isList: topmostNode == self
            )
        }

        return UserSelectedRelays(
            locations: locations,
            customListSelection: customListSelection
        )
    }

    func countryFor(code: String) -> LocationNode? {
        self.code == code ? self : children.first(where: { $0.code == code })
    }

    func cityFor(codes: [String]) -> LocationNode? {
        let combinedCode = Self.combineNodeCodes(codes)
        return self.code == combinedCode ? self : children.first(where: { $0.code == combinedCode })
    }

    func hostFor(code: String) -> LocationNode? {
        self.code == code ? self : children.first(where: { $0.code == code })
    }

    func descendantNodeFor(codes: [String]) -> LocationNode? {
        let combinedCode = Self.combineNodeCodes(codes)
        return self.code == combinedCode ? self : children.compactMap { $0.descendantNodeFor(codes: codes) }.first
    }

    func forEachDescendant(do callback: (LocationNode) -> Void) {
        children.forEach { child in
            callback(child)
            child.forEachDescendant(do: callback)
        }
    }

    func forEachAncestor(do callback: (LocationNode) -> Void) {
        if let parent = parent {
            callback(parent)
            parent.forEachAncestor(do: callback)
        }
    }

    static func combineNodeCodes(_ codes: [String]) -> String {
        codes.joined(separator: "-")
    }

    var flattened: [LocationNode] {
        children + children.flatMap { $0.flattened }
    }

    var activeRelayNodes: [LocationNode] {
        ([self] + flattened).filter { !($0 is CustomListLocationNode) }
            .filter(\.self.isActive)
            .filter {
                switch $0.locations.first {
                case .hostname:
                    return true
                default:
                    return false
                }
            }
    }

    func pathToRoot() -> [String] {
        var path: [String] = [name]
        forEachAncestor { locationNode in
            path.insert(NSLocalizedString(locationNode.name, comment: ""), at: 0)
        }
        return path
    }

    fileprivate func recursivelyCopyChildren(withParent parent: LocationNode) -> [LocationNode] {
        children.map { $0.copy(withParent: parent) }
    }
}
extension LocationNode: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(code)
    }

    static func == (lhs: LocationNode, rhs: LocationNode) -> Bool {
        lhs.code == rhs.code
    }
}

extension LocationNode: Comparable {
    static func < (lhs: LocationNode, rhs: LocationNode) -> Bool {
        lhs.name.lowercased() < rhs.name.lowercased()
    }
}

extension Array where Element == LocationNode {
    func forEachNode(_ body: (LocationNode) -> Void) {
        for element in self {
            body(element)
            element.children.forEachNode(body)
        }
    }
}

/// Proxy class for building and/or searching node trees.
class RootLocationNode: LocationNode, @unchecked Sendable {
    init(name: String = "", code: String = "", children: [LocationNode] = []) {
        super.init(name: name, code: code, children: children)
    }
}

class CustomListLocationNode: LocationNode, @unchecked Sendable {
    let customList: CustomList

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        isActive: Bool = true,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false,
        customList: CustomList
    ) {
        self.customList = customList

        super.init(
            name: name,
            code: code,
            locations: locations,
            isActive: isActive,
            parent: parent,
            children: children,
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch
        )
    }

    /// Recursively copies a node, its parent and its descendants from another
    /// node (tree), with an optional custom root parent.
    override func copy(withParent parent: LocationNode? = nil) -> LocationNode {
        let node = CustomListLocationNode(
            name: name,
            code: code,
            locations: locations,
            isActive: isActive,
            parent: parent,
            children: [],
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch,
            customList: customList
        )

        node.children = recursivelyCopyChildren(withParent: node)

        return node
    }

}

class RecentLocationNode: LocationNode, @unchecked Sendable {
    let locationInfo: [String]?

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        isActive: Bool = true,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false,
        locationInfo: [String]?
    ) {
        self.locationInfo = locationInfo

        super.init(
            name: name,
            code: code,
            locations: locations,
            isActive: isActive,
            parent: parent,
            children: children,
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch
        )
    }
}
