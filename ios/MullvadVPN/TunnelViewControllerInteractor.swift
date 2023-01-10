//
//  TunnelViewControllerInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class TunnelViewControllerInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateDeviceState: ((DeviceState) -> Void)?
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnelStatus: TunnelStatus {
        return tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        return tunnelManager.deviceState
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] tunnelManager, tunnelStatus in
                self?.didUpdateTunnelStatus?(tunnelStatus)
            },
            didUpdateDeviceState: { [weak self] tunnelManager, deviceState in
                self?.didUpdateDeviceState?(deviceState)
            }
        )
        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    func startTunnel() {
        tunnelManager.startTunnel()
    }

    func stopTunnel() {
        tunnelManager.stopTunnel()
    }

    func reconnectTunnel(selectNewRelay: Bool) {
        tunnelManager.reconnectTunnel(selectNewRelay: selectNewRelay)
    }
}
