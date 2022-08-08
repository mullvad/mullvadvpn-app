//
//  TunnelObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 19/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol TunnelObserver: AnyObject {
    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager)
    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus)
    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState)

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    )

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error)
}
