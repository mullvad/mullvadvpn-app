//
//  DeviceCheckRemoteService.swift
//  PacketTunnel
//
//  Created by pronebird on 30/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import WireGuardKitTypes

/// An object that implements remote service used by `DeviceCheckOperation`.
struct DeviceCheckRemoteService: DeviceCheckRemoteServiceProtocol {
    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling

    init(accountsProxy: RESTAccountHandling, devicesProxy: DeviceHandling) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    func getAccountData(
        accountNumber: String,
        completion: @escaping (Result<Account, Error>) -> Void
    ) -> Cancellable {
        accountsProxy.getAccountData(accountNumber: accountNumber).execute(completionHandler: completion)
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        devicesProxy.getDevice(
            accountNumber: accountNumber,
            identifier: identifier,
            retryStrategy: .noRetry,
            completion: completion
        )
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        devicesProxy.rotateDeviceKey(
            accountNumber: accountNumber,
            identifier: identifier,
            publicKey: publicKey,
            retryStrategy: .default,
            completion: completion
        )
    }
}
