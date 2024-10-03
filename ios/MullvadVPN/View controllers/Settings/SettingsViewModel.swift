//
//  SettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

struct SettingsViewModel {
    private(set) var daitaSettings: DAITASettings

    mutating func setDAITASettings(_ newSettings: DAITASettings) {
        daitaSettings = newSettings
    }

    init(from tunnelSettings: LatestTunnelSettings = LatestTunnelSettings()) {
        daitaSettings = tunnelSettings.daita
    }
}
