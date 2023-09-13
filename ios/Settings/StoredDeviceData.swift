//
//  StoredDeviceData.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import WireGuardKitTypes

public struct StoredDeviceData: Codable, Equatable {
    /// Device creation date.
    public var creationDate: Date

    /// Device identifier.
    public var identifier: String

    /// Device name.
    public var name: String

    /// Whether relay hijacks DNS from this device.
    public var hijackDNS: Bool

    /// IPv4 address + mask assigned to device.
    public var ipv4Address: IPAddressRange

    /// IPv6 address + mask assigned to device.
    public var ipv6Address: IPAddressRange

    /// WireGuard key data.
    public var wgKeyData: StoredWgKeyData

    /// Returns capitalized device name.
    public var capitalizedName: String {
        name.capitalized
    }

    public init(
        creationDate: Date,
        identifier: String,
        name: String,
        hijackDNS: Bool,
        ipv4Address: IPAddressRange,
        ipv6Address: IPAddressRange,
        wgKeyData: StoredWgKeyData
    ) {
        self.creationDate = creationDate
        self.identifier = identifier
        self.name = name
        self.hijackDNS = hijackDNS
        self.ipv4Address = ipv4Address
        self.ipv6Address = ipv6Address
        self.wgKeyData = wgKeyData
    }

    /// Fill in part of the structure that contains device related properties from `Device` struct.
    public mutating func update(from device: Device) {
        identifier = device.id
        name = device.name
        creationDate = device.created
        hijackDNS = device.hijackDNS
        ipv4Address = device.ipv4Address
        ipv6Address = device.ipv6Address
    }
}
