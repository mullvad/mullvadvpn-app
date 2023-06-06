//
//  RESTTypes.swift
//  MullvadTypes
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PublicKey

public struct Account: Codable, Equatable {
    public let id: String
    public let expiry: Date
    public let maxPorts: Int
    public let canAddPorts: Bool
    public let maxDevices: Int
    public let canAddDevices: Bool

    public init(id: String, expiry: Date, maxPorts: Int, canAddPorts: Bool, maxDevices: Int, canAddDevices: Bool) {
        self.id = id
        self.expiry = expiry
        self.maxPorts = maxPorts
        self.canAddPorts = canAddPorts
        self.maxDevices = maxDevices
        self.canAddDevices = canAddDevices
    }
}

public struct Device: Codable, Equatable {
    public let id: String
    public let name: String
    public let pubkey: PublicKey
    public let hijackDNS: Bool
    public let created: Date
    public let ipv4Address: IPAddressRange
    public let ipv6Address: IPAddressRange
    public let ports: [Port]

    private enum CodingKeys: String, CodingKey {
        case hijackDNS = "hijackDns"
        case id, name, pubkey, created, ipv4Address, ipv6Address, ports
    }

    public init(
        id: String,
        name: String,
        pubkey: PublicKey,
        hijackDNS: Bool,
        created: Date,
        ipv4Address: IPAddressRange,
        ipv6Address: IPAddressRange,
        ports: [Port]
    ) {
        self.id = id
        self.name = name
        self.pubkey = pubkey
        self.hijackDNS = hijackDNS
        self.created = created
        self.ipv4Address = ipv4Address
        self.ipv6Address = ipv6Address
        self.ports = ports
    }
}

public struct Port: Codable, Equatable {
    public let id: String

    public init(id: String) {
        self.id = id
    }
}
