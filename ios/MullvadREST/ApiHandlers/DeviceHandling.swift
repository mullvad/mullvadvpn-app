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
import MullvadTypes

public protocol DeviceHandling: Sendable {
    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable

    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy
    ) async throws -> Device

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<[Device]>
    ) -> Cancellable

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async throws -> [Device]

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy
    ) async throws -> Device

    func deleteDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Bool>
    ) -> Cancellable

    func deleteDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy
    ) async throws -> Bool

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        retryStrategy: REST.RetryStrategy,
    ) async throws -> Device
}
