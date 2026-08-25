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
import MullvadTypes

/// An object that implements remote service used by `DeviceCheckOperation`.
struct DeviceCheckRemoteService: DeviceCheckRemoteServiceProtocol {
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling

    init(accountsProxy: RESTAccountHandling, devicesProxy: DeviceHandling) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    func getAccountData(accountNumber: String) async -> Result<Account, Error> {
        await accountsProxy.getAccountData(
            accountNumber: accountNumber,
            retryStrategy: .noRetry
        )
    }

    func getDevice(
        accountNumber: String,
        identifier: String
    ) async -> Result<Device, Error> {
        await devicesProxy.getDevice(
            accountNumber: accountNumber,
            identifier: identifier,
            retryStrategy: .noRetry
        )
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey
    ) async -> Result<Device, Error> {
        await devicesProxy.rotateDeviceKey(
            accountNumber: accountNumber,
            identifier: identifier,
            publicKey: publicKey,
            retryStrategy: .default
        )
    }
}
