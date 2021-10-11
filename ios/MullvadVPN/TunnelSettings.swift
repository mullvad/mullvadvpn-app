//
//  TunnelSettings.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKit.PublicKey
import struct WireGuardKit.IPAddressRange

/// A struct that holds a tun interface configuration.
struct InterfaceSettings: Codable, Equatable {
    var privateKey: PrivateKeyWithMetadata
    var addresses: [IPAddressRange]
    var dnsSettings: DNSSettings

    var publicKey: PublicKey {
        return privateKey.publicKeyWithMetadata.publicKey
    }

    private enum CodingKeys: String, CodingKey {
        case privateKey, addresses, dnsSettings
    }

    init(privateKey: PrivateKeyWithMetadata = PrivateKeyWithMetadata(), addresses: [IPAddressRange] = [], dnsSettings: DNSSettings = DNSSettings()) {
        self.privateKey = privateKey
        self.addresses = addresses
        self.dnsSettings = dnsSettings
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        privateKey = try container.decode(PrivateKeyWithMetadata.self, forKey: .privateKey)
        addresses = try container.decode([IPAddressRange].self, forKey: .addresses)

        // Provide default value, since `dnsSettings` key does not exist in <= 2021.2
        dnsSettings = try container.decodeIfPresent(DNSSettings.self, forKey: .dnsSettings)
            ?? DNSSettings()
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(privateKey, forKey: .privateKey)
        try container.encode(addresses, forKey: .addresses)
        try container.encode(dnsSettings, forKey: .dnsSettings)
    }
}

/// A struct that holds the configuration passed via `NETunnelProviderProtocol`.
struct TunnelSettings: Codable, Equatable {
    var relayConstraints = RelayConstraints()
    var interface = InterfaceSettings()
}

/// A struct that holds DNS settings.
struct DNSSettings: Codable, Equatable {
    /// Maximum number of allowed DNS domains.
    static let maxAllowedCustomDNSDomains = 3

    /// Block advertising.
    var blockAdvertising: Bool = false

    /// Block tracking.
    var blockTracking: Bool = false

    /// Enable custom DNS.
    var enableCustomDNS: Bool = false

    /// Custom DNS domains.
    var customDNSDomains: [AnyIPAddress] = []

    /// Effective state of the custom DNS setting.
    var effectiveEnableCustomDNS: Bool {
        return !blockAdvertising && !blockTracking && enableCustomDNS && !customDNSDomains.isEmpty
    }

    private enum CodingKeys: String, CodingKey {
        case blockAdvertising, blockTracking, enableCustomDNS, customDNSDomains
    }

    init() {}

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        blockAdvertising = try container.decode(Bool.self, forKey: .blockAdvertising)
        blockTracking = try container.decode(Bool.self, forKey: .blockTracking)

        if let storedEnableCustomDNS = try container.decodeIfPresent(Bool.self, forKey: .enableCustomDNS) {
            enableCustomDNS = storedEnableCustomDNS
        }

        if let storedCustomDNSDomains = try container.decodeIfPresent([AnyIPAddress].self, forKey: .customDNSDomains) {
            customDNSDomains = storedCustomDNSDomains
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(blockAdvertising, forKey: .blockAdvertising)
        try container.encode(blockTracking, forKey: .blockTracking)
        try container.encode(enableCustomDNS, forKey: .enableCustomDNS)
        try container.encode(customDNSDomains, forKey: .customDNSDomains)
    }
}
