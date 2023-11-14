//
//  TunnelSettingsV1.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Network
import WireGuardKitTypes

/// A struct that holds the configuration passed via `NETunnelProviderProtocol`.
public struct TunnelSettingsV1: Codable, Equatable, TunnelSettings {
    public var relayConstraints = RelayConstraints()
    public var interface = InterfaceSettings()

    public func upgradeToNextVersion() -> any TunnelSettings {
        TunnelSettingsV2(relayConstraints: relayConstraints, dnsSettings: interface.dnsSettings)
    }
}

/// A struct that holds a tun interface configuration.
public struct InterfaceSettings: Codable, Equatable {
    public var privateKey: PrivateKeyWithMetadata
    public var nextPrivateKey: PrivateKeyWithMetadata?

    public var addresses: [IPAddressRange]
    public var dnsSettings: DNSSettings

    private enum CodingKeys: String, CodingKey {
        case privateKey, nextPrivateKey, addresses, dnsSettings
    }

    public init(
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

    public init(from decoder: Decoder) throws {
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

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(privateKey, forKey: .privateKey)
        try container.encode(nextPrivateKey, forKey: .nextPrivateKey)
        try container.encode(addresses, forKey: .addresses)
        try container.encode(dnsSettings, forKey: .dnsSettings)
    }
}

/// A struct holding a private WireGuard key with associated metadata
public struct PrivateKeyWithMetadata: Equatable, Codable {
    private enum CodingKeys: String, CodingKey {
        case privateKey = "privateKeyData", creationDate
    }

    /// When the key was created
    public let creationDate: Date

    /// Private key
    public let privateKey: PrivateKey

    /// Public key
    public var publicKey: PublicKey {
        privateKey.publicKey
    }

    /// Initialize the new private key
    public init() {
        privateKey = PrivateKey()
        creationDate = Date()
    }

    /// Initialize with the existing private key
    public init(privateKey: PrivateKey, createdAt: Date) {
        self.privateKey = privateKey
        creationDate = createdAt
    }
}
