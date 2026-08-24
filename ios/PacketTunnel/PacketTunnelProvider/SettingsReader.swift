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
import MullvadSettings
import PacketTunnelCore

struct SettingsReader: SettingsReaderProtocol {
    private let settingsManager: SettingsManager
    init(settingsManager: SettingsManager) {
        self.settingsManager = settingsManager
    }
    func read() throws -> Settings {
        let settings = try settingsManager.readSettings()
        let deviceState = try settingsManager.readDeviceState()
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
