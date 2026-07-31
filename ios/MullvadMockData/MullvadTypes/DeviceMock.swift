//
//  DeviceMock.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-03-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

extension Device {
    public static func mock(publicKey: WireGuard.PublicKey) -> Device {
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

    public static var loggedInDeviceState: DeviceState {
        .loggedIn(
            StoredAccountData(
                identifier: "",
                number: "",
                expiry: .distantFuture
            ),
            StoredDeviceData(
                creationDate: Date(),
                identifier: "",
                name: "",
                hijackDNS: false,
                ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
                ipv6Address: IPAddressRange(from: "::ff/64")!,
                wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: WireGuard.PrivateKey())
            )
        )
    }
}
