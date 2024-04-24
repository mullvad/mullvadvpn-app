//
//  RelaySelection.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct RelaySelection {
    var selected: UserSelectedRelays?
    var excluded: UserSelectedRelays?
    var excludedTitle: String?

    var hasExcludedRelay: Bool {
        if excluded?.locations.count == 1, case .hostname = excluded?.locations.first {
            return true
        }
        return false
    }
}
