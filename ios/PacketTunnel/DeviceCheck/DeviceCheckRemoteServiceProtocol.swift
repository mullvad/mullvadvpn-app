//
//  DeviceCheckRemoteServiceProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 07/06/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// A protocol that formalizes remote service dependency used by `DeviceCheckOperation`.
protocol DeviceCheckRemoteServiceProtocol {
    func getAccountData(accountNumber: String, completion: @escaping @Sendable (Result<Account, Error>) -> Void)
        -> Cancellable
    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping @Sendable (Result<Device, Error>) -> Void
    )
        -> Cancellable
    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        completion: @escaping @Sendable (Result<Device, Error>) -> Void
    ) -> Cancellable
}
