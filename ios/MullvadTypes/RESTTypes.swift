//
//  RESTTypes.swift
//  MullvadTypes
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct WireGuardKitTypes.IPAddressRange

/// Type holding account related data returned from REST API.
public struct AccountData: Codable, Equatable {
    public let id: String
    public let expiry: Date
    public let maxPorts: Int
    public let canAddPorts: Bool
    public let maxDevices: Int
    public let canAddDevices: Bool
}

public struct Device: Codable, Equatable {
    public let id: String
    public let name: String
    public let pubkey: String
    public let hijackDNS: Bool
    public let created: Date
    public let ipv4Address: IPAddressRange
    public let ipv6Address: IPAddressRange
    public let ports: [Port]

    private enum CodingKeys: String, CodingKey {
        case hijackDNS = "hijackDns"
        case id, name, pubkey, created, ipv4Address, ipv6Address, ports
    }
}

public struct Port: Codable, Equatable {
    public let id: String
}
