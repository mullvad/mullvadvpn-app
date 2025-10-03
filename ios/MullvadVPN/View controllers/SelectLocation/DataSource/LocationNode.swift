//
//  LocationNode.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

class LocationNode: @unchecked Sendable {
    let name: String
    var code: String
    var locations: [RelayLocation]
    var isActive: Bool
    weak var parent: LocationNode?
    var children: [LocationNode]
    var showsChildren: Bool
    var isHiddenFromSearch: Bool

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        isActive: Bool = true,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false
    ) {
        self.name = name
        self.code = code
        self.locations = locations
        self.isActive = isActive
        self.parent = parent
        self.children = children
        self.showsChildren = showsChildren
        self.isHiddenFromSearch = isHiddenFromSearch
    }
}

extension LocationNode {
    var root: LocationNode {
        parent?.root ?? self
    }

    var hierarchyLevel: Int {
        var level = 0
        forEachAncestor { _ in level += 1 }
        return level
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
}

extension LocationNode {
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
            isHiddenFromSearch: isHiddenFromSearch
        )

        node.children = recursivelyCopyChildren(withParent: node)

        return node
    }

    private func recursivelyCopyChildren(withParent parent: LocationNode) -> [LocationNode] {
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
}
