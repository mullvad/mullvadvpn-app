//
//  TunnelSettingsV1.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct Network.IPv4Address
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/// A struct that holds the configuration passed via `NETunnelProviderProtocol`.
struct TunnelSettingsV1: Codable, Equatable {
    var relayConstraints = RelayConstraints()
    var interface = InterfaceSettings()
}

/// A struct that holds a tun interface configuration.
struct InterfaceSettings: Codable, Equatable {
    var privateKey: PrivateKeyWithMetadata
    var nextPrivateKey: PrivateKeyWithMetadata?

    var addresses: [IPAddressRange]
    var dnsSettings: DNSSettings

    private enum CodingKeys: String, CodingKey {
        case privateKey, nextPrivateKey, addresses, dnsSettings
    }

    init(
        privateKey: PrivateKeyWithMetadata = PrivateKeyWithMetadata(),
        nextPrivateKey: PrivateKeyWithMetadata? = nil,
        addresses: [IPAddressRange] = [],
        dnsSettings: DNSSettings = DNSSettings()
    ) {
        self.privateKey = privateKey
        self.nextPrivateKey = nextPrivateKey
        self.addresses = addresses
        self.dnsSettings = dnsSettings
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        privateKey = try container.decode(PrivateKeyWithMetadata.self, forKey: .privateKey)
        addresses = try container.decode([IPAddressRange].self, forKey: .addresses)

        // Added in 2022.1
        nextPrivateKey = try container.decodeIfPresent(
            PrivateKeyWithMetadata.self,
            forKey: .nextPrivateKey
        )

        // Provide default value, since `dnsSettings` key does not exist in <= 2021.2
        dnsSettings = try container.decodeIfPresent(DNSSettings.self, forKey: .dnsSettings)
            ?? DNSSettings()
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(privateKey, forKey: .privateKey)
        try container.encode(nextPrivateKey, forKey: .nextPrivateKey)
        try container.encode(addresses, forKey: .addresses)
        try container.encode(dnsSettings, forKey: .dnsSettings)
    }
}

/// A struct holding a private WireGuard key with associated metadata
struct PrivateKeyWithMetadata: Equatable, Codable {
    private enum CodingKeys: String, CodingKey {
        case privateKey = "privateKeyData", creationDate
    }

    /// When the key was created
    let creationDate: Date

    /// Private key
    let privateKey: PrivateKey

    /// Public key
    var publicKey: PublicKey {
        return privateKey.publicKey
    }

    /// Initialize the new private key
    init() {
        privateKey = PrivateKey()
        creationDate = Date()
    }

    /// Initialize with the existing private key
    init(privateKey: PrivateKey, createdAt: Date) {
        self.privateKey = privateKey
        creationDate = createdAt
    }
}
