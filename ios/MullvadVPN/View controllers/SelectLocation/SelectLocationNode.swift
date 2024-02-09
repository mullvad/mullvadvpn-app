//
//  SelectLocationNode.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

enum LocationNodeType {
    case root
    case country
    case city
    case relay
}

class SelectLocationNode: SelectLocationNodeProtocol {
    var children: [SelectLocationNode]
    var showsChildren: Bool
    var nodeType: LocationNodeType
    var location: RelayLocation
    var displayName: String
    var isActive: Bool

    init(
        nodeType: LocationNodeType,
        location: RelayLocation,
        displayName: String = "",
        isActive: Bool = true,
        showsChildren: Bool = false,
        children: [SelectLocationNode] = []
    ) {
        self.showsChildren = showsChildren
        self.nodeType = nodeType
        self.location = location
        self.displayName = displayName
        self.isActive = isActive
        self.children = children
    }

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

    func addChild(_ child: SelectLocationNode) {
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
                node.isActive
            })
        case .relay:
            break
        }
    }

    func flatRelayLocationList(includeHiddenChildren: Bool = false) -> [RelayLocation] {
        children.reduce(into: []) { array, node in
            Self.flatten(node: node, into: &array, includeHiddenChildren: includeHiddenChildren)
        }
    }

    private func sortChildren() {
        switch nodeType {
        case .root, .country:
            children.sort { a, b -> Bool in
                a.displayName.localizedCaseInsensitiveCompare(b.displayName) == .orderedAscending
            }
        case .city:
            children.sort { a, b -> Bool in
                a.location.stringRepresentation
                    .localizedStandardCompare(b.location.stringRepresentation) == .orderedAscending
            }
        case .relay:
            break
        }
    }

    private class func flatten(
        node: SelectLocationNode,
        into array: inout [RelayLocation],
        includeHiddenChildren: Bool
    ) {
        array.append(node.location)
        if includeHiddenChildren || node.showsChildren {
            for child in node.children {
                Self.flatten(node: child, into: &array, includeHiddenChildren: includeHiddenChildren)
            }
        }
    }
}
