// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
