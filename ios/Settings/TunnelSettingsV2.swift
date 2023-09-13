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

public struct TunnelSettingsV2: Codable, Equatable {
    /// Relay constraints.
    public var relayConstraints: RelayConstraints

    /// DNS settings.
    public var dnsSettings: DNSSettings

    public init(
        relayConstraints: RelayConstraints = RelayConstraints(),
        dnsSettings: DNSSettings = DNSSettings()
    ) {
        self.relayConstraints = relayConstraints
        self.dnsSettings = dnsSettings
    }
}

public struct StoredWgKeyData: Codable, Equatable {
    /// Private key creation date.
    public var creationDate: Date

    /// Last date a rotation was attempted. Nil if last attempt was successful.
    public var lastRotationAttemptDate: Date?

    /// Private key.
    public var privateKey: PrivateKey

    /// Next private key we're trying to rotate to.
    /// Added in 2023.3
    public var nextPrivateKey: PrivateKey?

    public init(
        creationDate: Date,
        lastRotationAttemptDate: Date? = nil,
        privateKey: PrivateKey,
        nextPrivateKey: PrivateKey? = nil
    ) {
        self.creationDate = creationDate
        self.lastRotationAttemptDate = lastRotationAttemptDate
        self.privateKey = privateKey
        self.nextPrivateKey = nextPrivateKey
    }
}
