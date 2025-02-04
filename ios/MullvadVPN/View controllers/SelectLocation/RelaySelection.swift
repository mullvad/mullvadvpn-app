//
//  RelaySelection.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct RelaySelection: Sendable {
    var selected: UserSelectedRelays?
    var excluded: UserSelectedRelays?
    var excludedTitle: String?
}
