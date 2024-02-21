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
    var nodes: [LocationNode] { get }
    var searchableNodes: [LocationNode] { get }
}

extension LocationDataSourceProtocol {
    func search(by text: String) -> [LocationNode] {
        guard !text.isEmpty else {
            return nodes
        }

        var filteredNodes: [LocationNode] = []
        searchableNodes.forEach { countryNode in
            countryNode.showsChildren = false

            if countryNode.nodeName.fuzzyMatch(text) {
                filteredNodes.append(countryNode)
            }

            countryNode.children.forEach { cityNode in
                cityNode.showsChildren = false

                let relaysContainSearchString = cityNode.children
                    .contains(where: { $0.nodeName.fuzzyMatch(text) })

                if cityNode.nodeName.fuzzyMatch(text) || relaysContainSearchString {
                    if !filteredNodes.contains(countryNode) {
                        filteredNodes.append(countryNode)
                    }

                    countryNode.showsChildren = true

                    if relaysContainSearchString {
                        cityNode.showsChildren = true
                    }
                }
            }
        }

        return filteredNodes
    }
}
