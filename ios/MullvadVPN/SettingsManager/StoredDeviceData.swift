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

struct StoredDeviceData: Codable, Equatable {
    /// Device creation date.
    var creationDate: Date

    /// Device identifier.
    var identifier: String

    /// Device name.
    var name: String

    /// Whether relay hijacks DNS from this device.
    var hijackDNS: Bool

    /// IPv4 address + mask assigned to device.
    var ipv4Address: IPAddressRange

    /// IPv6 address + mask assigned to device.
    var ipv6Address: IPAddressRange

    /// WireGuard key data.
    var wgKeyData: StoredWgKeyData

    /// Returns capitalized device name.
    var capitalizedName: String {
        name.capitalized
    }

    /// Fill in part of the structure that contains device related properties from `Device` struct.
    mutating func update(from device: Device) {
        identifier = device.id
        name = device.name
        creationDate = device.created
        hijackDNS = device.hijackDNS
        ipv4Address = device.ipv4Address
        ipv6Address = device.ipv6Address
    }
}
