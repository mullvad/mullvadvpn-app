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
import MullvadTypes
import NetworkExtension

public protocol EphemeralPeerNegotiating {
    func startNegotiation(
        devicePublicKey: WireGuard.PublicKey,
        presharedKey: WireGuard.PrivateKey,
        peerReceiver: any TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool

    func cancelKeyNegotiation()

    init()
}

/// Requests an ephemeral peer asynchronously.
public class EphemeralPeerNegotiator: EphemeralPeerNegotiating {
    required public init() {}

    var cancelToken: OpaquePointer?

    public func startNegotiation(
        devicePublicKey: WireGuard.PublicKey,
        presharedKey: WireGuard.PrivateKey,
        peerReceiver: any TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        let ephemeralPeerReceiver = Unmanaged.passUnretained(peerReceiver as! EphemeralPeerReceiver)
            .toOpaque()

        guard let tunnelHandle = try? peerReceiver.tunnelHandle() else {
            return false
        }

        let cancelToken = request_ephemeral_peer(
            devicePublicKey.rawValue.map { $0 },
            presharedKey.rawValue.map { $0 },
            ephemeralPeerReceiver,
            tunnelHandle,
            ephemeralPeerParams
        )
        guard let cancelToken else {
            return false
        }
        self.cancelToken = cancelToken
        return true
    }

    public func cancelKeyNegotiation() {
        guard let cancelToken else { return }
        cancel_ephemeral_peer_exchange(cancelToken)
        self.cancelToken = nil
    }

    deinit {
        guard let cancelToken else { return }
        drop_ephemeral_peer_exchange_token(cancelToken)
    }
}
