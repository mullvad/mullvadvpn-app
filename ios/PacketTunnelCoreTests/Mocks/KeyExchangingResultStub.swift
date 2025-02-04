//
//  KeyExchangingResultStub.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes

struct KeyExchangingResultStub: EphemeralPeerReceiving {
    var onFailure: (() -> Void)?
    var onReceivePostQuantumKey: ((PreSharedKey, PrivateKey, DaitaV2Parameters?) async -> Void)?
    var onReceiveEphemeralPeerPrivateKey: ((PrivateKey, DaitaV2Parameters?) async -> Void)?

    func receivePostQuantumKey(
        _ key: PreSharedKey,
        ephemeralKey: PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await onReceivePostQuantumKey?(key, ephemeralKey, daitaParameters)
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: PrivateKey,
        daitaParameters daitaParamters: MullvadTypes.DaitaV2Parameters?
    ) async {
        await onReceiveEphemeralPeerPrivateKey?(ephemeralPeerPrivateKey, daitaParamters)
    }

    func ephemeralPeerExchangeFailed() {
        onFailure?()
    }
}
