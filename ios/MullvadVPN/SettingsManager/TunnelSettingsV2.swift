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
}
