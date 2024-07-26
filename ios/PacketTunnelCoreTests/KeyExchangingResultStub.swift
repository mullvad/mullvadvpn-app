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

struct KeyExchangingResultStub: PostQuantumKeyReceiving {
    var onFailure: (() -> Void)?
    var onReceivePostQuantumKey: ((PreSharedKey, PrivateKey) -> Void)?

    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        onReceivePostQuantumKey?(key, ephemeralKey)
    }

    func keyExchangeFailed() {
        onFailure?()
    }
}
