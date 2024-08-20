//
//  KeyExchangingResultStub.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes

struct KeyExchangingResultStub: EphemeralPeerReceiving {
    var onFailure: (() -> Void)?
    var onReceivePostQuantumKey: ((PreSharedKey, PrivateKey) -> Void)?
    var onReceiveEphemeralPeerPrivateKey: ((PrivateKey) -> Void)?

    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        onReceivePostQuantumKey?(key, ephemeralKey)
    }

    public func receiveEphemeralPeerPrivateKey(_ ephemeralPeerPrivateKey: PrivateKey) {
        onReceiveEphemeralPeerPrivateKey?(ephemeralPeerPrivateKey)
    }

    func ephemeralPeerExchangeFailed() {
        onFailure?()
    }
}
