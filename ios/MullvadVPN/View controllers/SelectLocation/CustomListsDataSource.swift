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
        return []
    }

    func reload(
        _ response: MullvadREST.REST.ServerRelaysResponse,
        relays: [MullvadREST.REST.ServerRelay]
    ) -> [RelayLocation] {
        locationList
    }
}
