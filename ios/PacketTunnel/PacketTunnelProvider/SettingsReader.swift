//
//  SettingsReader.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import PacketTunnelCore

struct SettingsReader: SettingsReaderProtocol {
    func read() throws -> Settings {
        let settings = try SettingsManager.readSettings()
        let deviceState = try SettingsManager.readDeviceState()
        let deviceData = try deviceState.getDeviceData()

        return Settings(
            privateKey: deviceData.wgKeyData.privateKey,
            interfaceAddresses: [deviceData.ipv4Address, deviceData.ipv6Address],
            tunnelSettings: settings
        )
    }
}

private extension DeviceState {
    /**
     Returns `StoredDeviceState` if device is logged in, otherwise throws an error.

     - Throws: an error of type `ReadDeviceDataError` when device is either revoked or logged out.
     - Returns: a copy of `StoredDeviceData` stored as associated value in `DeviceState.loggedIn` variant.
     */
    func getDeviceData() throws -> StoredDeviceData {
        switch self {
        case .revoked:
            throw ReadDeviceDataError.revoked
        case .loggedOut:
            throw ReadDeviceDataError.loggedOut
        case let .loggedIn(_, deviceData):
            return deviceData
        }
    }
}

/// Error returned when device state is either revoked or logged out.
public enum ReadDeviceDataError: LocalizedError {
    case loggedOut, revoked

    public var errorDescription: String? {
        switch self {
        case .loggedOut:
            return "Device is logged out."
        case .revoked:
            return "Device is revoked."
        }
    }
}
