//
//  PostQuantumConfiguration.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import WireGuardKitTypes

public enum PostQuantumNegotiationState: Equatable {
    case single(PostQuantumConfigurationRelay)
    case multi(entry: PostQuantumConfigurationRelay, exit: PostQuantumConfigurationRelay)

    public static func == (lhs: PostQuantumNegotiationState, rhs: PostQuantumNegotiationState) -> Bool {
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

public struct PostQuantumConfigurationRelay: Equatable, CustomDebugStringConvertible {
    public let relay: SelectedRelay
    public let configuration: PostQuantumConfiguration

    public init(relay: SelectedRelay, configuration: PostQuantumConfiguration) {
        self.relay = relay
        self.configuration = configuration
    }

    public var debugDescription: String {
        "{ relay : \(relay.debugDescription), post quantum: \(configuration.debugDescription) }"
    }
}

public struct PostQuantumConfiguration: Equatable, CustomDebugStringConvertible {
    public let privateKey: PrivateKey
    public let preSharedKey: PreSharedKey?
    public let allowedIPs: [IPAddressRange]

    public init(privateKey: PrivateKey, preSharedKey: PreSharedKey? = nil, allowedIPs: [IPAddressRange]) {
        self.privateKey = privateKey
        self.preSharedKey = preSharedKey
        self.allowedIPs = allowedIPs
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
