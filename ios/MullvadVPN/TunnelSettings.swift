//
//  TunnelSettings.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import NetworkExtension
import WireGuardKit

/// A struct that holds a tun interface configuration.
struct InterfaceSettings: Codable {
    var privateKey: PrivateKeyWithMetadata
    var addresses: [IPAddressRange]
    var dnsSettings: DNSSettings

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
struct TunnelSettings: Codable {
    var relayConstraints = RelayConstraints()
    var interface = InterfaceSettings()
}

/// A struct that holds DNS settings.
struct DNSSettings: Codable {
    /// Block advertising.
    var blockAdvertising: Bool = false

    /// Block tracking.
    var blockTracking: Bool = false
}
