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
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations

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

    func updateAccountData() async -> Result<Void, Error> {
        guard case let .loggedIn(accountData, _) = deviceState else {
            return .failure(InvalidDeviceStateError())
        }

        let result = await accountsProxy.getAccountData(
            accountNumber: accountData.number,
            retryStrategy: .default
        )

        return result.tryMap { accountData in
            switch deviceState {
            case .loggedIn(var storedAccountData, let storedDeviceData):
                storedAccountData.expiry = accountData.expiry
                let newDeviceState = DeviceState.loggedIn(storedAccountData, storedDeviceData)

                // Make sure we don't update any data if cancellation happened in-flight.
                if Task.isCancelled {
                    throw CancellationError()
                } else {
                    tunnelManager.setDeviceState(newDeviceState, persist: true)
                }
            default:
                throw InvalidDeviceStateError()
            }
        }
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }

    #if NEVER_IN_PRODUCTION
        /// Replaces the device key with one that was never published to the API, then reconnects the tunnel if it
        /// is up. This enables testing if the packet tunnel can recover a bad key.
        func invalidateWireGuardKey() {
            guard case .loggedIn(let accountData, var deviceData) = deviceState else { return }

            deviceData.wgKeyData = StoredWgKeyData(
                creationDate: deviceData.wgKeyData.creationDate,
                privateKey: WireGuard.PrivateKey()
            )
            tunnelManager.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

            if tunnelManager.tunnelStatus.state.isSecured {
                tunnelManager.reconnectTunnel(selectNewRelay: false)
            }
        }
    #endif
}
