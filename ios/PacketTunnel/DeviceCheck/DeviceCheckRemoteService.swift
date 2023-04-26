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
import class WireGuardKitTypes.PublicKey

/// An object that implements remote service used by `CheckDeviceOperation`.
struct DeviceCheckRemoteService: DeviceCheckRemoteServiceProtocol {
    private let accountsProxy: REST.AccountsProxy
    private let devicesProxy: REST.DevicesProxy

    init(accountsProxy: REST.AccountsProxy, devicesProxy: REST.DevicesProxy) {
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
    }

    func getAccountData(
        accountNumber: String,
        completion: @escaping (Result<AccountData, Error>) -> Void
    ) -> Cancellable {
        accountsProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .noRetry, completion: completion)
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

/// An object that implements access to `DeviceState`.
struct DeviceStateAccessor: DeviceStateAccessorProtocol {
    func read() throws -> DeviceState {
        return try SettingsManager.readDeviceState()
    }

    func write(_ deviceState: DeviceState) throws {
        try SettingsManager.writeDeviceState(deviceState)
    }
}
