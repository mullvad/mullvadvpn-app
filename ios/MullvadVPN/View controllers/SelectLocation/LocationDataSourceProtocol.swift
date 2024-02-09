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
    var nodeByLocation: [RelayLocation: SelectLocationNode] { get }

    func search(by text: String) -> [RelayLocation]

    func reload(
        _ response: REST.ServerRelaysResponse,
        relays: [REST.ServerRelay]
    ) -> [RelayLocation]
}

extension LocationDataSourceProtocol {
    func makeRootNode(name: String) -> SelectLocationNode {
        SelectLocationNode(nodeType: .root, location: .country("#root"), displayName: name)
    }

    func createNode(
        root: SelectLocationNode,
        ascendantOrSelf: RelayLocation,
        serverLocation: REST.ServerLocation,
        relay: REST.ServerRelay,
        wasShowingChildren: Bool
    ) -> SelectLocationNode {
        let node: SelectLocationNode

        switch ascendantOrSelf {
        case .country:
            node = SelectLocationNode(
                nodeType: .country,
                location: ascendantOrSelf,
                displayName: serverLocation.country,
                showsChildren: wasShowingChildren
            )
            root.addChild(node)
        case let .city(countryCode, _):
            node = SelectLocationNode(
                nodeType: .city,
                location: ascendantOrSelf,
                displayName: serverLocation.city,
                showsChildren: wasShowingChildren
            )
            nodeByLocation[.country(countryCode)]!.addChild(node)

        case let .hostname(countryCode, cityCode, _):
            node = SelectLocationNode(
                nodeType: .relay,
                location: ascendantOrSelf,
                displayName: relay.hostname,
                isActive: relay.active
            )
            nodeByLocation[.city(countryCode, cityCode)]!.addChild(node)
        }
        return node
    }
}
