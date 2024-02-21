//
//  LocationCellViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct LocationCellViewModel: Hashable {
    let group: SelectLocationSection
    let location: RelayLocation
}
