//
//  EphemeralPeerReceiver.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-15.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy
import MullvadTypes
import NetworkExtension
import WireGuardKitTypes

/// End sequence of an ephemeral peer exchange.
///
/// This FFI function is called by Rust when an ephemeral peer negotiation succeeded or failed.
/// When both the `rawPresharedKey` and the `rawEphemeralKey` are raw pointers to 32 bytes data arrays,
/// the quantum-secure key exchange is considered successful.
/// If the `rawPresharedKey` is nil, but there is a valid `rawEphemeralKey`, it means a Daita peer has been negotiated with.
/// If `rawEphemeralKey` is nil, the negotiation is considered failed.
///
/// - Parameters:
///   - rawEphemeralPeerReceiver: A raw pointer to the running instance of `NEPacketTunnelProvider`
///   - rawPresharedKey: A raw pointer to the quantum-secure pre shared key
///   - rawEphemeralKey: A raw pointer to the ephemeral private key of the device
@_cdecl("swift_ephemeral_peer_ready")
func receivePostQuantumKey(
    rawEphemeralPeerReceiver: UnsafeMutableRawPointer?,
    rawPresharedKey: UnsafeMutableRawPointer?,
    rawEphemeralKey: UnsafeMutableRawPointer?
) {
    guard let rawEphemeralPeerReceiver else { return }
    let ephemeralPeerReceiver = Unmanaged<EphemeralPeerReceiver>.fromOpaque(rawEphemeralPeerReceiver)
        .takeUnretainedValue()

    // If there are no private keys for the ephemeral peer, then the negotiation either failed, or timed out.
    guard let rawEphemeralKey,
          let ephemeralKey = PrivateKey(rawValue: Data(bytes: rawEphemeralKey, count: 32)) else {
        ephemeralPeerReceiver.ephemeralPeerExchangeFailed()
        return
    }

    // If there is a pre-shared key, an ephemeral peer was negotiated with Post Quantum options
    // Otherwise, a Daita enabled ephemeral peer was requested
    if let rawPresharedKey, let key = PreSharedKey(rawValue: Data(bytes: rawPresharedKey, count: 32)) {
        ephemeralPeerReceiver.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    } else {
        ephemeralPeerReceiver.receiveEphemeralPeerPrivateKey(ephemeralKey)
    }
    return
}
