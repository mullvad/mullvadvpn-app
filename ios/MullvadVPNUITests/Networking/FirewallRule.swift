//
//  FirewallRule.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

enum NetworkingProtocol: String {
    case TCP = "tcp"
    case UDP = "udp"
    case ICMP = "icmp"
}

struct FirewallRule {
    let fromIPAddress: String
    let toIPAddress: String
    let protocols: [NetworkingProtocol]

    /// - Parameters:
    ///     - fromIPAddress: Block traffic originating from this source IP address.
    ///     - toIPAddress: Block traffic to this destination IP address.
    ///     - protocols: Protocols which should be blocked. If none is specified all will be blocked.
    private init(fromIPAddress: String, toIPAddress: String, protocols: [NetworkingProtocol]) {
        self.fromIPAddress = fromIPAddress
        self.toIPAddress = toIPAddress
        self.protocols = protocols
    }

    /// Make a firewall rule blocking API access for the current device under test
    public static func makeBlockAPIAccessFirewallRule() throws -> FirewallRule {
        let deviceIPAddress = try Networking.getIPAddress()
        let apiIPAddress = try MullvadAPIWrapper.getAPIIPAddress()
        return FirewallRule(
            fromIPAddress: deviceIPAddress,
            toIPAddress: apiIPAddress,
            protocols: [NetworkingProtocol.TCP]
        )
    }
}
