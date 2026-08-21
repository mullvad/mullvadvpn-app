// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings

struct SettingsViewModel {
    private(set) var daitaSettings: DAITASettings
    private(set) var multihopState: MultihopState
    private(set) var includeAllNetworksState: InclueAllNetworksState

    var currentLanguage: String {
        let currentLanguage = ApplicationLanguage.currentLanguage
        return currentLanguage.displayName
    }

    init(from tunnelSettings: LatestTunnelSettings = LatestTunnelSettings()) {
        daitaSettings = tunnelSettings.daita
        multihopState = tunnelSettings.tunnelMultihopState
        includeAllNetworksState = tunnelSettings.includeAllNetworks.includeAllNetworksState
    }
}
