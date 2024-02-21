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
    var indentationLevel: Int
    var parent: LocationNode?
    var children: [LocationNode]
    var showsChildren: Bool

    init(
        nodeName: String,
        nodeCode: String,
        locations: [RelayLocation] = [],
        indentationLevel: Int = 0,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false
    ) {
        self.nodeName = nodeName
        self.nodeCode = nodeCode
        self.locations = locations
        self.indentationLevel = indentationLevel
        self.parent = parent
        self.children = children
        self.showsChildren = showsChildren
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

    func nodeFor(nodeCode: String) -> LocationNode? {
        self.nodeCode == nodeCode ? self : children.compactMap { $0.nodeFor(nodeCode: nodeCode) }.first
    }

    func forEachDescendant(do callback: (_ index: Int, _ node: LocationNode) -> Void) {
        children.enumerated().forEach { index, node in
            callback(index, node)
            node.forEachDescendant(do: callback)
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
            indentationLevel: indentationLevel,
            parent: parent,
            children: [],
            showsChildren: showsChildren
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

/// Dummy class for building and/or searching node trees.
class RootNode: LocationNode {
    init(nodeName: String = "", nodeCode: String = "", children: [LocationNode] = []) {
        super.init(nodeName: nodeName, nodeCode: nodeCode, children: children)
    }
}

class LocationListNode: LocationNode {
    let customList: CustomList

    init(
        nodeName: String,
        nodeCode: String,
        locations: [RelayLocation] = [],
        indentationLevel: Int = 0,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        customList: CustomList
    ) {
        self.customList = customList

        super.init(
            nodeName: nodeName,
            nodeCode: nodeCode,
            locations: locations,
            indentationLevel: indentationLevel,
            parent: parent,
            children: children,
            showsChildren: showsChildren
        )
    }
}

class LocationCountryNode: LocationNode {}

class LocationCityNode: LocationNode {}

class LocationHostNode: LocationNode {}
