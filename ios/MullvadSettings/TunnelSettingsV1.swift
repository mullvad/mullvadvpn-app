//
//  TunnelSettingsV1.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

/// A struct that holds the configuration passed via `NETunnelProviderProtocol`.
public struct TunnelSettingsV1: Codable, Equatable, TunnelSettings {
    public var relayConstraints = RelayConstraints()
    public var dnsSettings = DNSSettings()

    public func upgradeToNextVersion() -> any TunnelSettings {
        return TunnelSettingsV2(relayConstraints: relayConstraints, dnsSettings: dnsSettings)
    }

    public var debugDescription: String {
        "TunnelSettingsV1(relayConstraints: \(relayConstraints))"
    }
}

/// A struct that holds a tun interface configuration.
public struct InterfaceSettings: Codable, Equatable, CustomDebugStringConvertible, @unchecked Sendable {
    public var privateKey: PrivateKeyWithMetadata
    public var nextPrivateKey: PrivateKeyWithMetadata?

    public var addresses: [IPAddressRange]
    public var dnsSettings: DNSSettings

    private enum CodingKeys: String, CodingKey {
        case privateKey, nextPrivateKey, addresses, dnsSettings
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
        dnsSettings =
            try container.decodeIfPresent(DNSSettings.self, forKey: .dnsSettings)
            ?? DNSSettings()
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(privateKey, forKey: .privateKey)
        try container.encode(nextPrivateKey, forKey: .nextPrivateKey)
        try container.encode(addresses, forKey: .addresses)
        try container.encode(dnsSettings, forKey: .dnsSettings)
    }

    public var debugDescription: String {
        "dnsSettings: \(dnsSettings), addresses: \(addresses)"
    }
}

/// A struct holding a private WireGuard key with associated metadata
public struct PrivateKeyWithMetadata: Equatable, Codable {
    private enum CodingKeys: String, CodingKey {
        case privateKey = "privateKeyData"
        case creationDate
    }

    /// When the key was created
    public let creationDate: Date

    /// Private key
    public let privateKey: WireGuard.PrivateKey

    /// Initialize with the existing private key
    public init(privateKey: WireGuard.PrivateKey, createdAt: Date) {
        self.privateKey = privateKey
        creationDate = createdAt
    }
}
