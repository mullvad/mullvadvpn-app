//
//  SettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

struct SettingsViewModel {
    private(set) var daitaSettings: DAITASettings
    private(set) var multihopState: MultihopState

    var currentLanguage: String {
        let currentLanguage = AppLanguage.currentLanguage
        return "\(currentLanguage.flagEmoji) \(currentLanguage.displayName)"
    }

    init(from tunnelSettings: LatestTunnelSettings = LatestTunnelSettings()) {
        daitaSettings = tunnelSettings.daita
        multihopState = tunnelSettings.tunnelMultihopState
    }
}
