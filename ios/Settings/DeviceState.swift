//
//  DeviceState.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum DeviceState: Codable, Equatable {
    case loggedIn(StoredAccountData, StoredDeviceData)
    case loggedOut
    case revoked

    private enum LoggedInCodableKeys: String, CodingKey {
        case _0 = "account"
        case _1 = "device"
    }

    public var isLoggedIn: Bool {
        switch self {
        case .loggedIn:
            return true
        case .loggedOut, .revoked:
            return false
        }
    }

    public var accountData: StoredAccountData? {
        switch self {
        case let .loggedIn(accountData, _):
            return accountData
        case .loggedOut, .revoked:
            return nil
        }
    }

    public var deviceData: StoredDeviceData? {
        switch self {
        case let .loggedIn(_, deviceData):
            return deviceData
        case .loggedOut, .revoked:
            return nil
        }
    }
}
