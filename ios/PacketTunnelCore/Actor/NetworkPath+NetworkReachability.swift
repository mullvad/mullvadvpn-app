//
//  NetworkPath+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

extension Network.NWPath.Status {
    /// Converts `NetworkPath.status` into `NetworkReachability`.
    var networkReachability: NetworkReachability {
        switch self {
        case .satisfied:
            .reachable
        case .unsatisfied:
            .unreachable
        case .requiresConnection:
            .reachable
        @unknown default:
            .undetermined
        }
    }
}

extension Network.NWPath {
    public var unsatisfiedReasonDescription: String {
        switch unsatisfiedReason {
        case .cellularDenied: "User has disabled cellular"
        case .localNetworkDenied: "User has disabled local network access"
        case .notAvailable: "Not available, no reason given"
        case .vpnInactive: "Required VPN, but no active VPN found"
        case .wifiDenied: "User has disabled Wifi"
        @unknown default: "Unknown situation"
        }
    }
}

extension NWInterface {
    public var customDebugDescription: String {
        "type: \(type) name: \(self.name) index: \(index)"
    }
}

extension NWEndpoint.Host {
    public var customDebugDescription: String {
        switch self {
        case let .ipv4(IPv4Address): "IPv4: \(IPv4Address)"
        case let .ipv6(IPv6Address): "IPv6: \(IPv6Address)"
        case let .name(name, interface): "named: \(name), \(interface?.customDebugDescription ?? "[No interface]")"
        @unknown default: "Unknown host"
        }
    }
}

extension NWInterface.InterfaceType: @retroactive CustomDebugStringConvertible {
    public var debugDescription: String {
        switch self {
        case .cellular: "Cellular"
        case .loopback: "Loopback"
        case .other: "Other"
        case .wifi: "Wifi"
        case .wiredEthernet: "Wired Ethernet"
        @unknown default: "Unknown interface type"
        }
    }
}

extension NWEndpoint {
    public var customDebugDescription: String {
        switch self {
        case let .hostPort(host, port): "host: \(host.customDebugDescription) port: \(port)"
        case let .opaque(endpoint): "opaque: \(endpoint.description)"
        case let .url(url): "url: \(url)"
        case let .service(
            name,
            type,
            domain,
            interface
        ): "service named:\(name), type:\(type), domain:\(domain), interface:\(interface?.customDebugDescription ?? "[No interface]")"
        case let .unix(path): "unix: \(path)"
        @unknown default: "Unknown NWEndpoint type"
        }
    }
}
