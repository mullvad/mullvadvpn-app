//
//  MullvadPostQuantum+Stubs.swift
//  MullvadPostQuantumTests
//
//  Created by Marco Nikic on 2024-06-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadPostQuantum
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

class FailedNegotiatorStub: PostQuantumKeyNegotiation {
    var onCancelKeyNegotiation: (() -> Void)?

    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: WireGuardKitTypes.PublicKey,
        presharedKey: WireGuardKitTypes.PrivateKey,
        packetTunnel: PacketTunnelCore.TunnelProvider,
        tcpConnection: NWTCPConnection,
        postQuantumKeyExchangeTimeout: MullvadTypes.Duration
    ) -> Bool { false }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}

class SuccessfulNegotiatorStub: PostQuantumKeyNegotiation {
    var onCancelKeyNegotiation: (() -> Void)?
    required init() {
        onCancelKeyNegotiation = nil
    }

    init(onCancelKeyNegotiation: (() -> Void)? = nil) {
        self.onCancelKeyNegotiation = onCancelKeyNegotiation
    }

    func startNegotiation(
        gatewayIP: IPv4Address,
        devicePublicKey: WireGuardKitTypes.PublicKey,
        presharedKey: WireGuardKitTypes.PrivateKey,
        packetTunnel: PacketTunnelCore.TunnelProvider,
        tcpConnection: NWTCPConnection,
        postQuantumKeyExchangeTimeout: MullvadTypes.Duration
    ) -> Bool { true }

    func cancelKeyNegotiation() {
        onCancelKeyNegotiation?()
    }
}

// swiftlint:enable function_parameter_count
