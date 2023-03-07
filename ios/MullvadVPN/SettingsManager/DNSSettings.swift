//
//  DNSSettings.swift
//  MullvadVPN
//
//  Created by pronebird on 27/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct Network.IPv4Address

/// A struct describing Mullvad DNS blocking options.
struct DNSBlockingOptions: OptionSet, Codable {
    typealias RawValue = UInt32

    let rawValue: RawValue

    static let blockAdvertising = DNSBlockingOptions(rawValue: 1 << 0)
    static let blockTracking = DNSBlockingOptions(rawValue: 1 << 1)
    static let blockMalware = DNSBlockingOptions(rawValue: 1 << 2)
    static let blockAdultContent = DNSBlockingOptions(rawValue: 1 << 3)
    static let blockGambling = DNSBlockingOptions(rawValue: 1 << 4)

    var serverAddress: IPv4Address? {
        if isEmpty {
            return nil
        } else {
            return IPv4Address("100.64.0.\(rawValue)")
        }
    }

    init(rawValue: RawValue) {
        self.rawValue = rawValue
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let rawValue = try container.decode(RawValue.self)

        self.init(rawValue: rawValue)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(rawValue)
    }
}

/// A struct that holds DNS settings.
struct DNSSettings: Codable, Equatable {
    /// Maximum number of allowed DNS domains.
    static let maxAllowedCustomDNSDomains = 3

    /// DNS blocking options.
    var blockingOptions: DNSBlockingOptions = []

    /// Enable custom DNS.
    var enableCustomDNS = false

    /// Custom DNS domains.
    var customDNSDomains: [AnyIPAddress] = []

    /// Effective state of the custom DNS setting.
    var effectiveEnableCustomDNS: Bool {
        return blockingOptions.isEmpty && enableCustomDNS && !customDNSDomains.isEmpty
    }

    private enum CodingKeys: String, CodingKey {
        // Removed in 2022.1 in favor of `blockingOptions`
        case blockAdvertising, blockTracking

        // Added in 2022.1
        case blockingOptions

        case enableCustomDNS, customDNSDomains
    }

    init() {}

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Added in 2022.1
        if let storedBlockingOptions = try container.decodeIfPresent(
            DNSBlockingOptions.self,
            forKey: .blockingOptions
        ) {
            blockingOptions = storedBlockingOptions
        }

        if let storedBlockAdvertising = try container.decodeIfPresent(
            Bool.self,
            forKey: .blockAdvertising
        ), storedBlockAdvertising {
            blockingOptions.insert(.blockAdvertising)
        }

        if let storedBlockTracking = try container.decodeIfPresent(
            Bool.self,
            forKey: .blockTracking
        ), storedBlockTracking {
            blockingOptions.insert(.blockTracking)
        }

        if let storedEnableCustomDNS = try container.decodeIfPresent(
            Bool.self,
            forKey: .enableCustomDNS
        ) {
            enableCustomDNS = storedEnableCustomDNS
        }

        if let storedCustomDNSDomains = try container.decodeIfPresent(
            [AnyIPAddress].self,
            forKey: .customDNSDomains
        ) {
            customDNSDomains = storedCustomDNSDomains
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(blockingOptions, forKey: .blockingOptions)
        try container.encode(enableCustomDNS, forKey: .enableCustomDNS)
        try container.encode(customDNSDomains, forKey: .customDNSDomains)
    }
}
