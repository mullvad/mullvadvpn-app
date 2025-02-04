//
//  CustomListViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

struct CustomListViewModel {
    var id: UUID
    var name: String
    var locations: [RelayLocation]
    let tableSections: [CustomListSectionIdentifier]

    var customList: CustomList {
        CustomList(id: id, name: name, locations: locations)
    }

    mutating func update(with list: CustomList) {
        name = list.name
        locations = list.locations
    }
}
