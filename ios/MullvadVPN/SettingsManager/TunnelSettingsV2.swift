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

    /// Returns `true` if account has expired.
    var isExpired: Bool {
        return expiry <= Date()
    }
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

    /**
     Mutates account and device data when in logged in state, otherwise does nothing.
     */
    mutating func updateData(_ body: (inout StoredAccountData, inout StoredDeviceData) -> Void) {
        switch self {
        case var .loggedIn(accountData, deviceData):
            body(&accountData, &deviceData)
            self = .loggedIn(accountData, deviceData)
        case .revoked, .loggedOut:
            break
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

    var capitalizedName: String {
        name.capitalized
    }
}

struct StoredWgKeyData: Codable, Equatable {
    /// Private key creation date.
    var creationDate: Date

    /// Last date a rotation was attempted. Nil if last attempt was successful.
    var lastRotationAttemptDate: Date?

    /// Private key.
    var privateKey: PrivateKey

    /// Next private key we're trying to rotate to.
    /// Added in 2023.3
    var nextPrivateKey: PrivateKey?

    /// Returns next private key if set, otherwise creates new key, assigns next private key and then returns it.
    mutating func getOrCreateNextPrivateKey() -> PrivateKey {
        if let nextPrivateKey = nextPrivateKey {
            return nextPrivateKey
        } else {
            let newKey = PrivateKey()
            nextPrivateKey = newKey
            return newKey
        }
    }

    /// Assigns last rotation attempt to current date.
    mutating func markRotationAttempt() {
        lastRotationAttemptDate = Date()
    }
}

extension StoredWgKeyData {
    struct KeyRotationConfiguration {
        let rotationInterval: TimeInterval = 60 * 60 * 24 * 14
        let retryInterval: TimeInterval = 60 * 60 * 24
    }

    func getNextRotationDate(for configuration: KeyRotationConfiguration) -> Date {
        return max(
            Date(),
            lastRotationAttemptDate?.addingTimeInterval(configuration.retryInterval) ?? creationDate
                .addingTimeInterval(configuration.rotationInterval)
        )
    }

    /**
     Returns `true` if packet tunnel should perform key rotation.

     During the startup packet tunnel rotates the key immediately if it detected that the key stored on server does not
     match the key stored on device. In that case it passes `shouldRotateImmediately = true`.

     To dampen the effect of packet tunnel entering into a restart cycle and going on a key rotation rampage,
     this function adds a cooldown interval to prevent it from pushing keys too often.
     */
    func shouldPacketTunnelRotateTheKey(shouldRotateImmediately: Bool) -> Bool {
        guard let lastRotationAttemptDate = lastRotationAttemptDate else { return true }

        let retryInterval: TimeInterval = 60 * 60 * 24
        let cooldownInterval: TimeInterval = 15

        let now = Date()
        let nextRotationAttempt = max(now, lastRotationAttemptDate.addingTimeInterval(retryInterval))

        if nextRotationAttempt <= now {
            return true
        }

        // Add cooldown interval when requested to rotate the key immediately.
        if shouldRotateImmediately, lastRotationAttemptDate.distance(to: now) > cooldownInterval {
            return true
        }

        return false
    }
}
