//
//  DeviceHandling.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
@preconcurrency import WireGuardKitTypes

public protocol DeviceHandling: Sendable {
    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<[Device]>
    ) -> Cancellable

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable

    func deleteDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Bool>
    ) -> Cancellable

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Device>
    ) -> Cancellable
}
