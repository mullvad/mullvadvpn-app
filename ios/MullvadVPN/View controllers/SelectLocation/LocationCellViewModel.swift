//
//  LocationCellViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct LocationCellViewModel: Hashable {
    let section: LocationSection
    let node: LocationNode
    var indentationLevel = 0

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.node == rhs.node &&
            lhs.section == rhs.section
    }
}
