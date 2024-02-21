//
//  CustomListViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
}
