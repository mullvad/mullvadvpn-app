//
//  PostQuantumKeyReceiving.swift
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
    ///   - key: The preshared key used by the Post Quantum Peer
    ///   - ephemeralKey: The private key used by the Post Quantum Peer
    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey)

    func receiveEphemeralPeerPrivateKey(_: PrivateKey)

    /// Called when an ephemeral peer could not be successfully negotiated
    func ephemeralPeerExchangeFailed()
}
