//
//  MullvadPostQuantum+Stubs.swift
//  MullvadRustRuntimeTests
//
//  Created by Marco Nikic on 2024-06-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadRustRuntime
@testable import MullvadTypes
import NetworkExtension
@testable import PacketTunnelCore
@testable import WireGuardKitTypes

// swiftlint:disable function_parameter_count
class NWTCPConnectionStub: NWTCPConnection {
    var _isViable = false
    override var isViable: Bool {
        _isViable
    }

    func becomeViable() {
        willChangeValue(for: \.isViable)
        _isViable = true
        didChangeValue(for: \.isViable)
    }
}

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

    let tcpConnection: NWTCPConnectionStub

    init(tcpConnection: NWTCPConnectionStub) {
        self.tcpConnection = tcpConnection
    }

    func createTCPConnectionThroughTunnel(
        to remoteEndpoint: NWEndpoint,
        enableTLS: Bool,
        tlsParameters TLSParameters: NWTLSParameters?,
        delegate: Any?
    ) -> NWTCPConnection {
        tcpConnection
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

// swiftlint:enable function_parameter_count
