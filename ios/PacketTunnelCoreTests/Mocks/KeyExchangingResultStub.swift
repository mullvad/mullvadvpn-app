//
//  KeyExchangingResultStub.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
