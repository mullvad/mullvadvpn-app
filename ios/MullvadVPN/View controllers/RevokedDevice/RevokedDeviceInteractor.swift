//
//  RevokedDeviceInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class RevokedDeviceInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didUpdateTunnelStatus?(tunnelStatus)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}
