// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadSettings

protocol TunnelObserver: AnyObject, Sendable {
    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager)
    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus)
    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateDeviceState deviceState: DeviceState,
        previousDeviceState: DeviceState
    )
    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: LatestTunnelSettings)
    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error)
}
