//
//  CustomListsDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

class CustomListsDataSource: LocationDataSourceProtocol {
    var nodeByLocation = [RelayLocation: SelectLocationNode]()
    private var locationList = [RelayLocation]()

    func search(by text: String) -> [RelayLocation] {
        []
    }

    func reload(
        _ response: REST.ServerRelaysResponse,
        relays: [REST.ServerRelay]
    ) -> [RelayLocation] {
        locationList
    }
}
