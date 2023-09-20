//
//  TunnelViewControllerInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

final class TunnelViewControllerInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateDeviceState: ((_ deviceState: DeviceState, _ previousDeviceState: DeviceState) -> Void)?
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didUpdateTunnelStatus?(tunnelStatus)
            },
            didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                self?.didUpdateDeviceState?(deviceState, previousDeviceState)
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
