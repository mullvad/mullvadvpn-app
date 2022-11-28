//
//  TunnelSettingsV2.swift
//  MullvadVPN
//
//  Created by pronebird on 27/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct Network.IPv4Address
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/// Settings and device state schema versions.
enum SchemaVersion: Int, Equatable {
    /// Legacy settings format, stored as `TunnelSettingsV1`.
    case v1 = 1

    /// New settings format, stored as `TunnelSettingsV2`.
    case v2 = 2

    /// Current schema version.
    static let current = SchemaVersion.v2
}

struct TunnelSettingsV2: Codable, Equatable {
    /// Relay constraints.
    var relayConstraints = RelayConstraints()

    /// DNS settings.
    var dnsSettings = DNSSettings()
}

struct StoredAccountData: Codable, Equatable {
    /// Account identifier.
    var identifier: String

    /// Account number.
    var number: String

    /// Account expiry.
    var expiry: Date
}

enum DeviceState: Codable, Equatable {
    case loggedIn(StoredAccountData, StoredDeviceData)
    case loggedOut
    case revoked

    private enum LoggedInCodableKeys: String, CodingKey {
        case _0 = "account"
        case _1 = "device"
    }

    var isLoggedIn: Bool {
        switch self {
        case .loggedIn:
            return true
        case .loggedOut, .revoked:
            return false
        }
    }

    var accountData: StoredAccountData? {
        switch self {
        case let .loggedIn(accountData, _):
            return accountData
        case .loggedOut, .revoked:
            return nil
        }
    }

    var deviceData: StoredDeviceData? {
        switch self {
        case let .loggedIn(_, deviceData):
            return deviceData
        case .loggedOut, .revoked:
            return nil
        }
    }
}

struct StoredDeviceData: Codable, Equatable {
    /// Device creation date.
    var creationDate: Date

    /// Device identifier.
    var identifier: String

    /// Device name.
    var name: String

    /// Whether relay hijacks DNS from this device.
    var hijackDNS: Bool

    /// IPv4 address assigned to device.
    var ipv4Address: IPAddressRange

    /// IPv6 address assignged to device.
    var ipv6Address: IPAddressRange

    /// WireGuard key data.
    var wgKeyData: StoredWgKeyData
}

struct StoredWgKeyData: Codable, Equatable {
    /// Private key creation date.
    var creationDate: Date

    /// Private key.
    var privateKey: PrivateKey
}
