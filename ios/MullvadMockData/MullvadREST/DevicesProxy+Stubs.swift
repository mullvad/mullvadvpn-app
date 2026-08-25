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

struct DevicesProxyStubError: Error {}

struct DevicesProxyStub: DeviceHandling {
    var deviceResult: Result<Device, Error> = .failure(DevicesProxyStubError())

    func getDevice(accountNumber: String, identifier: String, retryStrategy: REST.RetryStrategy) async -> Result<
        Device, any Error
    > {
        deviceResult
    }

    func getDevices(accountNumber: String, retryStrategy: REST.RetryStrategy) async -> Result<[Device], any Error> {
        switch deviceResult {
        case let .success(success):
            .success([success])
        case let .failure(failure):
            .failure(failure)
        }
    }

    func createDevice(accountNumber: String, request: CreateDeviceRequest, retryStrategy: REST.RetryStrategy) async
        -> Result<Device, any Error>
    {
        deviceResult
    }

    func deleteDevice(accountNumber: String, identifier: String, retryStrategy: REST.RetryStrategy) async -> Result<
        Bool, any Error
    > {
        .success(true)
    }

    func rotateDeviceKey(
        accountNumber: String, identifier: String, publicKey: WireGuard.PublicKey, retryStrategy: REST.RetryStrategy
    ) async -> Result<Device, any Error> {
        deviceResult
    }
}
