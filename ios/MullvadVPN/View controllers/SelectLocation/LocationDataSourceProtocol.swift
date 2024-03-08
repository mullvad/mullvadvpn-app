//
//  LocationDataSourceProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import UIKit

protocol LocationDataSourceProtocol {
    var viewForHeader: UIView? { get }
    var heightForFooterInSection: CGFloat { get }
    var nodes: [LocationNode] { get }
    var searchableNodes: [LocationNode] { get }
}

extension LocationDataSourceProtocol {
    func search(by text: String) -> [LocationNode] {
        guard !text.isEmpty else {
            resetNodes()
            return nodes
        }

        var filteredNodes: [LocationNode] = []

        searchableNodes.forEach { countryNode in
            countryNode.showsChildren = false

            if countryNode.name.fuzzyMatch(text) {
                filteredNodes.append(countryNode)
            }

            countryNode.children.forEach { cityNode in
                cityNode.showsChildren = false
                cityNode.isHiddenFromSearch = true

                var relaysContainSearchString = false
                cityNode.children.forEach { hostNode in
                    hostNode.isHiddenFromSearch = true

                    if hostNode.name.fuzzyMatch(text) {
                        relaysContainSearchString = true
                        hostNode.isHiddenFromSearch = false
                    }
                }

                if cityNode.name.fuzzyMatch(text) || relaysContainSearchString {
                    if !filteredNodes.contains(countryNode) {
                        filteredNodes.append(countryNode)
                    }

                    countryNode.showsChildren = true
                    cityNode.isHiddenFromSearch = false

                    if relaysContainSearchString {
                        cityNode.showsChildren = true
                    }
                }
            }
        }

        return filteredNodes
    }

    private func resetNodes() {
        nodes.forEach { node in
            node.showsChildren = false
            node.isHiddenFromSearch = false

            node.forEachDescendant { descendantNode in
                descendantNode.showsChildren = false
                descendantNode.isHiddenFromSearch = false
            }
        }
    }
}
