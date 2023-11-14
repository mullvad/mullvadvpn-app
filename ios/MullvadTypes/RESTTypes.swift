//
//  RESTTypes.swift
//  MullvadTypes
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

public struct Account: Codable, Equatable {
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

public struct Device: Codable, Equatable {
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
