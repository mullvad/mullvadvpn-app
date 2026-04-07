//
//  EphemeralPeerNegotiationState.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes

@preconcurrency import class WireGuardKitTypes.PreSharedKey
@preconcurrency import class WireGuardKitTypes.PrivateKey

public enum EphemeralPeerNegotiationState: Equatable, Sendable {
    case single(EphemeralPeerRelayConfiguration)
    case multi(entry: EphemeralPeerRelayConfiguration, exit: EphemeralPeerRelayConfiguration)

    var ephemeralPeerKeys: (entry: WireGuard.PrivateKey?, exit: WireGuard.PrivateKey) {
        switch self {
        case .single(let configuration):
            (nil, configuration.configuration.privateKey)
        case .multi(let entryConfiguration, let exitConfiguration):
            (entryConfiguration.configuration.privateKey, exitConfiguration.configuration.privateKey)
        }
    }

    public static func == (lhs: EphemeralPeerNegotiationState, rhs: EphemeralPeerNegotiationState) -> Bool {
        return switch (lhs, rhs) {
        case let (.single(hop1), .single(hop2)):
            hop1 == hop2
        case let (.multi(entry: entry1, exit: exit1), .multi(entry: entry2, exit: exit2)):
            entry1 == entry2 && exit1 == exit2
        default:
            false
        }
    }
}

public struct EphemeralPeerRelayConfiguration: Equatable, Sendable {
    public let relay: SelectedRelay
    public let configuration: EphemeralPeerConfiguration

    public init(relay: SelectedRelay, configuration: EphemeralPeerConfiguration) {
        self.relay = relay
        self.configuration = configuration
    }
}

public struct EphemeralPeerConfiguration: Equatable, Sendable {
    public let privateKey: WireGuard.PrivateKey
    public let preSharedKey: WireGuard.PreSharedKey?
    public let allowedIPs: [IPAddressRange]
    public let daitaParameters: DaitaV2Parameters?

    public init(
        privateKey: WireGuard.PrivateKey,
        preSharedKey: WireGuard.PreSharedKey? = nil,
        allowedIPs: [IPAddressRange],
        daitaParameters: DaitaV2Parameters?
    ) {
        self.privateKey = privateKey
        self.preSharedKey = preSharedKey
        self.allowedIPs = allowedIPs
        self.daitaParameters = daitaParameters
    }
}
