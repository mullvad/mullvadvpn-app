//
//  MullvadPostQuantum+Stubs.swift
//  MullvadRustRuntimeTests
//
//  Created by Marco Nikic on 2024-06-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import NetworkExtension

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore
@testable import WireGuardKitTypes

class TunnelProviderStub: TunnelProvider {
    func tunnelHandle() throws -> Int32 {
        0
    }

    func wgFunctions() -> MullvadTypes.WgFunctionPointers {
        return MullvadTypes.WgFunctionPointers(
            open: { _, _, _ in return 0 },
            close: { _, _ in return 0 },
            receive: { _, _, _, _ in return 0 },
            send: { _, _, _, _ in return 0 }
        )
    }
}

class FailedNegotiatorStub: EphemeralPeerNegotiating {
    var onCancelKeyNegotiation: (() -> Void)?

    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        devicePublicKey: WireGuardKitTypes.PublicKey,
        presharedKey: WireGuardKitTypes.PrivateKey,
        peerReceiver: any MullvadTypes.TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        false
    }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}

class SuccessfulNegotiatorStub: EphemeralPeerNegotiating {
    var onCancelKeyNegotiation: (() -> Void)?
    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        devicePublicKey: WireGuardKitTypes.PublicKey,
        presharedKey: WireGuardKitTypes.PrivateKey,
        peerReceiver: any MullvadTypes.TunnelProvider,
        ephemeralPeerParams: EphemeralPeerParameters
    ) -> Bool {
        true
    }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}
