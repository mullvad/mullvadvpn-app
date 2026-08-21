// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import NetworkExtension

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore

class TunnelProviderStub: TunnelProvider {
    func tunnelHandle() throws -> Int32 {
        0
    }

    func wgFunctions() -> MullvadTypes.WgFunctionPointers {
        return MullvadTypes.WgFunctionPointers(
            open: { _, _, _ in return 0 },
            close: { _, _ in return 0 },
            receive: { _, _, _, _ in return 0 },
            send: { _, _, _, _ in return 0 }
        )
    }
}

class FailedNegotiatorStub: EphemeralPeerNegotiating {
    var onCancelKeyNegotiation: (() -> Void)?

    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        devicePublicKey: WireGuard.PublicKey,
        presharedKey: WireGuard.PrivateKey,
        peerReceiver: any MullvadTypes.TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        false
    }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}

class SuccessfulNegotiatorStub: EphemeralPeerNegotiating {
    var onCancelKeyNegotiation: (() -> Void)?
    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        devicePublicKey: WireGuard.PublicKey,
        presharedKey: WireGuard.PrivateKey,
        peerReceiver: any MullvadTypes.TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        true
    }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}
