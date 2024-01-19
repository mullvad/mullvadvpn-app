//
//  FirewallRule.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class FirewallRule {
    let fromIPAddress: String
    let toIPAddress: String
    let protocols: [String]

    /// - Parameters:
    ///     - fromIPAddress: Block traffic originating from this source IP address.
    ///     - toIPAddress: Block traffic to this destination IP address.
    ///     - protocols: Protocols which should be blocked. Valid values: `tcp`, `udp`, `icmp`. If none is specified all will be blocked.
    private init(fromIPAddress: String, toIPAddress: String, protocols: [String]) {
        self.fromIPAddress = fromIPAddress
        self.toIPAddress = toIPAddress
        self.protocols = protocols
    }

    /// Make a firewall rule blocking API access for the current device under test
    public static func makeBlockAPIAccessFirewallRule() -> FirewallRule {
        if let deviceIPAddress = FirewallAPIClient.getIPAddress() {
            let apiIPAddress = MullvadAPIWrapper.endpoint.components(separatedBy: ":").first!
            return FirewallRule(fromIPAddress: deviceIPAddress, toIPAddress: apiIPAddress, protocols: ["tcp"])
        } else {
            fatalError("Failed to get IP address of device")
        }
    }
}
