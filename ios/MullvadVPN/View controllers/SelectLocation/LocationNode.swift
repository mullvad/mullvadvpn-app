//
//  LocationNode.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

class LocationNode {
    let name: String
    var code: String
    var locations: [RelayLocation]
    var parent: LocationNode?
    var children: [LocationNode]
    var showsChildren: Bool
    var isHiddenFromSearch: Bool

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false
    ) {
        self.name = name
        self.code = code
        self.locations = locations
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

    func countryFor(code: String) -> LocationNode? {
        self.code == code ? self : children.first(where: { $0.code == code })
    }

    func cityFor(code: String) -> LocationNode? {
        self.code == code ? self : children.first(where: { $0.code == code })
    }

    func hostFor(code: String) -> LocationNode? {
        self.code == code ? self : children.first(where: { $0.code == code })
    }

    func descendantNodeFor(code: String) -> LocationNode? {
        self.code == code ? self : children.compactMap { $0.descendantNodeFor(code: code) }.first
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
}

extension LocationNode {
    func copy(withParent parent: LocationNode? = nil) -> LocationNode {
        let node = LocationNode(
            name: name,
            code: code,
            locations: locations,
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
        lhs.name < rhs.name
    }
}

/// Proxy class for building and/or searching node trees.
class RootLocationNode: LocationNode {
    init(name: String = "", code: String = "", children: [LocationNode] = []) {
        super.init(name: name, code: code, children: children)
    }
}

class CustomListLocationNode: LocationNode {
    let customList: CustomList

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
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
            parent: parent,
            children: children,
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch
        )
    }
}

class CountryLocationNode: LocationNode {}

class CityLocationNode: LocationNode {}

class HostLocationNode: LocationNode {}
