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
    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        completion(deviceResult)
        return AnyCancellable()
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Device, Swift.Error> {
        deviceResult
    }

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<[Device]>
    ) -> Cancellable {
        switch deviceResult {
        case let .success(success):
            completion(.success([success]))
        case let .failure(failure):
            completion(.failure(failure))
        }
        return AnyCancellable()
    }

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<[Device], Swift.Error> {
        deviceResult.map { [$0] }
    }

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        completion(deviceResult)
        return AnyCancellable()
    }

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Device, Swift.Error> {
        deviceResult
    }

    func deleteDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Bool>
    ) -> Cancellable {
        completion(.success(true))
        return AnyCancellable()
    }

    func deleteDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Bool, Swift.Error> {
        .success(true)
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        completion(deviceResult)
        return AnyCancellable()
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        retryStrategy: REST.RetryStrategy,
    ) async -> Result<Device, Swift.Error> {
        deviceResult
    }
}
