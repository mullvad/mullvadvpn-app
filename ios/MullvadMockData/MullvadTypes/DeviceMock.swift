//
//  DeviceMock.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-03-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import WireGuardKitTypes

extension Device {
    public static func mock(publicKey: PublicKey) -> Device {
        Device(
            id: "device-id",
            name: "Secure Mole",
            pubkey: publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        )
    }
}
