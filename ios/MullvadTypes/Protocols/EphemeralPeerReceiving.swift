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

public protocol EphemeralPeerReceiving {
    /// Called when successfully requesting an ephemeral peer with Post Quantum PSK enabled
    ///
    /// - Parameters:
    ///   - key: The preshared key used by the Ephemeral Peer
    ///   - ephemeralKey: The private key used by the Ephemeral Peer
    ///   - daitaParameters: DAITA parameters
    func receivePostQuantumKey(
        _ key: WireGuard.PreSharedKey, ephemeralKey: WireGuard.PrivateKey, daitaParameters: DaitaV2Parameters?) async

    /// Called when successfully requesting an ephemeral peer with Daita enabled, and Post Quantum PSK disabled
    /// - Parameters:
    ///     - _: The private key used by the Ephemeral Peer
    ///     - daitaParameters: DAITA parameters
    func receiveEphemeralPeerPrivateKey(_: WireGuard.PrivateKey, daitaParameters: DaitaV2Parameters?) async

    /// Called when an ephemeral peer could not be successfully negotiated
    func ephemeralPeerExchangeFailed()
}
