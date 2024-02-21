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
    let nodeName: String
    var nodeCode: String
    var locations: [RelayLocation]
    var parent: LocationNode?
    var children: [LocationNode]
    var showsChildren: Bool
    var isHiddenFromSearch: Bool

    init(
        nodeName: String,
        nodeCode: String,
        locations: [RelayLocation] = [],
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false
    ) {
        self.nodeName = nodeName
        self.nodeCode = nodeCode
        self.locations = locations
        self.parent = parent
        self.children = children
        self.showsChildren = showsChildren
        self.isHiddenFromSearch = isHiddenFromSearch
    }
}

extension LocationNode {
    var topmostAncestor: LocationNode {
        parent?.topmostAncestor ?? self
    }

    func countryFor(countryCode: String) -> LocationNode? {
        nodeCode == countryCode ? self : children.first(where: { $0.nodeCode == countryCode })
    }

    func cityFor(cityCode: String) -> LocationNode? {
        nodeCode == cityCode ? self : children.first(where: { $0.nodeCode == cityCode })
    }

    func hostFor(hostCode: String) -> LocationNode? {
        nodeCode == hostCode ? self : children.first(where: { $0.nodeCode == hostCode })
    }

    func childNodeFor(nodeCode: String) -> LocationNode? {
        self.nodeCode == nodeCode ? self : children.compactMap { $0.childNodeFor(nodeCode: nodeCode) }.first
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
            nodeName: nodeName,
            nodeCode: nodeCode,
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
        hasher.combine(nodeCode)
    }

    static func == (lhs: LocationNode, rhs: LocationNode) -> Bool {
        lhs.nodeCode == rhs.nodeCode
    }
}

extension LocationNode: Comparable {
    static func < (lhs: LocationNode, rhs: LocationNode) -> Bool {
        lhs.nodeName < rhs.nodeName
    }
}

/// Proxy class for building and/or searching node trees.
class RootLocationNode: LocationNode {
    init(nodeName: String = "", nodeCode: String = "", children: [LocationNode] = []) {
        super.init(nodeName: nodeName, nodeCode: nodeCode, children: children)
    }
}

class ListLocationNode: LocationNode {
    let customList: CustomList

    init(
        nodeName: String,
        nodeCode: String,
        locations: [RelayLocation] = [],
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false,
        customList: CustomList
    ) {
        self.customList = customList

        super.init(
            nodeName: nodeName,
            nodeCode: nodeCode,
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
