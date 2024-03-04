//
//  DevicesProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
@testable import MullvadTypes
@testable import WireGuardKitTypes

struct DevicesProxyStub: DeviceHandling {
    let mockDevice = Device.mock(publicKey: PrivateKey().publicKey)
    func getDevice(
        accountNumber: String,
        identifier: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        AnyCancellable()
    }

    func getDevices(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<[Device]>
    ) -> Cancellable {
        AnyCancellable()
    }

    func createDevice(
        accountNumber: String,
        request: REST.CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        completion(.success(mockDevice))
        return AnyCancellable()
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

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        AnyCancellable()
    }
}
