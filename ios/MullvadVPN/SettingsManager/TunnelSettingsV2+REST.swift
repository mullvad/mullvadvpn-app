//
//  TunnelSettingsV2+REST.swift
//  MullvadVPN
//
//  Created by pronebird on 13/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import class WireGuardKitTypes.PrivateKey

extension StoredDeviceData {
    mutating func update(from device: REST.Device) {
        identifier = device.id
        name = device.name
        creationDate = device.created
        hijackDNS = device.hijackDNS
        ipv4Address = device.ipv4Address
        ipv6Address = device.ipv6Address
    }
}
