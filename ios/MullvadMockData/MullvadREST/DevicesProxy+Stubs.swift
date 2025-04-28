//
//  DevicesProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import WireGuardKitTypes

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

    func createDevice(
        accountNumber: String,
        request: CreateDeviceRequest,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Device>
    ) -> Cancellable {
        completion(deviceResult)
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
        completion(deviceResult)
        return AnyCancellable()
    }
}
