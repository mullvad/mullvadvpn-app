//
//  AccountInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import StoreKit

final class AccountInteractor: Sendable {
    let tunnelManager: TunnelManager
    let accountsProxy: RESTAccountHandling
    let apiProxy: APIQuerying
    let deviceProxy: DeviceHandling

    nonisolated(unsafe) var didReceiveTunnelState: (() -> Void)?
    nonisolated(unsafe) var didReceiveDeviceState: (@Sendable (DeviceState) -> Void)?

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?

    init(
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling,
        apiProxy: APIQuerying,
        deviceProxy: DeviceHandling
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.apiProxy = apiProxy
        self.deviceProxy = deviceProxy

        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, _ in
                    self?.didReceiveTunnelState?()
                },
                didUpdateDeviceState: { [weak self] _, deviceState, _ in
                    self?.didReceiveDeviceState?(deviceState)
                }
            )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    var tunnelState: TunnelState {
        tunnelManager.tunnelStatus.state
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }
}
