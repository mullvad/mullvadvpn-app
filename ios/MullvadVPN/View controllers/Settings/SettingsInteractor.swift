// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadSettings
import UIKit

final class SettingsInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    var didUpdateSettings: (() -> Void)?

    private(set) var tunnelSettings: LatestTunnelSettings
    private(set) var deviceState: DeviceState

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        self.tunnelSettings = tunnelManager.settings
        self.deviceState = tunnelManager.deviceState

        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateDeviceState: { [weak self] _, deviceState, _ in
                    guard let self = self else { return }
                    self.deviceState = deviceState
                    self.didUpdateSettings?()
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self = self else { return }
                    self.tunnelSettings = settings
                    self.didUpdateSettings?()
                }
            )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}
