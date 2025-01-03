//
//  EphemeralPeerNegotiationState.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
@preconcurrency import WireGuardKitTypes

public enum EphemeralPeerNegotiationState: Equatable, Sendable {
    case single(EphemeralPeerRelayConfiguration)
    case multi(entry: EphemeralPeerRelayConfiguration, exit: EphemeralPeerRelayConfiguration)

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

public struct EphemeralPeerRelayConfiguration: Equatable, CustomDebugStringConvertible, Sendable {
    public let relay: SelectedRelay
    public let configuration: EphemeralPeerConfiguration

    public init(relay: SelectedRelay, configuration: EphemeralPeerConfiguration) {
        self.relay = relay
        self.configuration = configuration
    }

    public var debugDescription: String {
        "{ relay : \(relay.debugDescription), post quantum: \(configuration.debugDescription) }"
    }
}

public struct EphemeralPeerConfiguration: Equatable, CustomDebugStringConvertible, Sendable {
    public let privateKey: PrivateKey
    public let preSharedKey: PreSharedKey?
    public let allowedIPs: [IPAddressRange]
    public let daitaParameters: DaitaV2Parameters?

    public init(
        privateKey: PrivateKey,
        preSharedKey: PreSharedKey? = nil,
        allowedIPs: [IPAddressRange],
        daitaParameters: DaitaV2Parameters?
    ) {
        self.privateKey = privateKey
        self.preSharedKey = preSharedKey
        self.allowedIPs = allowedIPs
        self.daitaParameters = daitaParameters
    }

    public var debugDescription: String {
        var string = "{ private key : \(privateKey),"
        string += preSharedKey.flatMap {
            "preShared key: \($0), "
        } ?? ""
        string += ", allowedIPs: \(allowedIPs) }"
        return string
    }
}
