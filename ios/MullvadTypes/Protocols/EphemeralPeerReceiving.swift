//
//  EphemeralPeerReceiving.swift
//  MullvadTypes
//
//  Created by Andrew Bulhak on 2024-03-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

public protocol EphemeralPeerReceiving {
    /// Called when successfully requesting an ephemeral peer with Post Quantum PSK enabled
    ///
    /// - Parameters:
    ///   - key: The preshared key used by the Ephemeral Peer
    ///   - ephemeralKey: The private key used by the Ephemeral Peer
    ///   - daitaParameters: DAITA parameters
    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey, daitaParameters: DaitaV2Parameters?) async

    /// Called when successfully requesting an ephemeral peer with Daita enabled, and Post Quantum PSK disabled
    /// - Parameters:
    ///     - _: The private key used by the Ephemeral Peer
    ///     - daitaParameters: DAITA parameters
    func receiveEphemeralPeerPrivateKey(_: PrivateKey, daitaParameters: DaitaV2Parameters?) async

    /// Called when an ephemeral peer could not be successfully negotiated
    func ephemeralPeerExchangeFailed()
}
