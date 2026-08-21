// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

@testable import MullvadRustRuntime
@testable import MullvadTypes

struct KeyExchangingResultStub: EphemeralPeerReceiving {
    var onFailure: (() -> Void)?
    var onReceivePostQuantumKey: ((WireGuard.PreSharedKey, WireGuard.PrivateKey, DaitaV2Parameters?) async -> Void)?
    var onReceiveEphemeralPeerPrivateKey: ((WireGuard.PrivateKey, DaitaV2Parameters?) async -> Void)?

    func receivePostQuantumKey(
        _ key: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await onReceivePostQuantumKey?(key, ephemeralKey, daitaParameters)
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: WireGuard.PrivateKey,
        daitaParameters: MullvadTypes.DaitaV2Parameters?
    ) async {
        await onReceiveEphemeralPeerPrivateKey?(ephemeralPeerPrivateKey, daitaParameters)
    }

    func ephemeralPeerExchangeFailed() {
        onFailure?()
    }
}
