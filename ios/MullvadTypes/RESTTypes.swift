//
//  RESTTypes.swift
//  MullvadTypes
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@preconcurrency import WireGuardKitTypes

public struct Account: Codable, Equatable, Sendable {
    public let id: String
    public let expiry: Date
    public let maxDevices: Int
    public let canAddDevices: Bool

    public init(id: String, expiry: Date, maxDevices: Int, canAddDevices: Bool) {
        self.id = id
        self.expiry = expiry
        self.maxDevices = maxDevices
        self.canAddDevices = canAddDevices
    }
}

public struct Device: Codable, Equatable, Sendable {
    public let id: String
    public let name: String
    public let pubkey: PublicKey
    public let hijackDNS: Bool
    public let created: Date
    public let ipv4Address: IPAddressRange
    public let ipv6Address: IPAddressRange

    private enum CodingKeys: String, CodingKey {
        case hijackDNS = "hijackDns"
        case id, name, pubkey, created, ipv4Address, ipv6Address
    }

    public init(
        id: String,
        name: String,
        pubkey: PublicKey,
        hijackDNS: Bool,
        created: Date,
        ipv4Address: IPAddressRange,
        ipv6Address: IPAddressRange
    ) {
        self.id = id
        self.name = name
        self.pubkey = pubkey
        self.hijackDNS = hijackDNS
        self.created = created
        self.ipv4Address = ipv4Address
        self.ipv6Address = ipv6Address
    }
}

public struct ProblemReportRequest: Codable, Sendable {
    public let address: String
    public let message: String
    public let log: String
    public let metadata: [String: String]

    public init(address: String, message: String, log: String, metadata: [String: String]) {
        self.address = address
        self.message = message
        self.log = log
        self.metadata = metadata
    }
}

public struct CreateDeviceRequest: Codable, Sendable {
    public let publicKey: PublicKey
    public let hijackDNS: Bool

    public init(publicKey: PublicKey, hijackDNS: Bool) {
        self.publicKey = publicKey
        self.hijackDNS = hijackDNS
    }

    private enum CodingKeys: String, CodingKey {
        case hijackDNS = "hijackDns"
        case publicKey = "pubkey"
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(publicKey.base64Key, forKey: .publicKey)
        try container.encode(hijackDNS, forKey: .hijackDNS)
    }
}

public struct RotateDeviceKeyRequest: Codable, Sendable {
    let publicKey: PublicKey

    public init(publicKey: PublicKey) {
        self.publicKey = publicKey
    }

    private enum CodingKeys: String, CodingKey {
        case publicKey = "pubkey"
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(publicKey.base64Key, forKey: .publicKey)
    }
}
