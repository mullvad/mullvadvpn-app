//
//  EphemeralPeerReceiving.swift
//  MullvadTypes
//
//  Created by Andrew Bulhak on 2024-03-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

public protocol EphemeralPeerReceiving {
    /// Called when successfully requesting an ephemeral peer with Post Quantum PSK enabled
    ///
    /// - Parameters:
    ///   - key: The preshared key used by the Ephemeral Peer
    ///   - ephemeralKey: The private key used by the Ephemeral Peer
    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) async

    /// Called when successfully requesting an ephemeral peer with Daita enabled, and Post Quantum PSK disabled
    /// - Parameter _:_ The private key used by the Ephemeral Peer
    func receiveEphemeralPeerPrivateKey(_: PrivateKey) async

    /// Called when an ephemeral peer could not be successfully negotiated
    func ephemeralPeerExchangeFailed()
}
