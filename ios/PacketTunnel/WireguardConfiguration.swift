//
//  WireguardConfiguration.swift
//  PacketTunnel
//
//  Created by pronebird on 17/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct describing a basic WireGuard configuration
struct WireguardConfiguration {
    var privateKey: WireguardPrivateKey
    var peers: [WireguardPeer]
    var allowedIPs: [IPAddressRange]
}

extension WireguardConfiguration {

    /// Returns commands suitable for configuring WireGuard
    func uapiConfiguration() -> [WireguardCommand] {
        var commands: [WireguardCommand] = [
            .privateKey(privateKey),
            .listenPort(0)
        ]

        commands.append(.replacePeers)
        peers.forEach { (peer) in
            commands.append(.peer(peer))
        }

        commands.append(.replaceAllowedIPs)
        allowedIPs.forEach { (ipAddressRange) in
            commands.append(.allowedIP(ipAddressRange))
        }

        return commands
    }

    /// Returns commands suitable for updating existing endpoints when roaming between networks
    /// (i.e Wi-Fi, cellular)
    func endpointUapiConfiguration() -> [WireguardCommand] {
        var commands: [WireguardCommand] = []

        peers.forEach { (peer) in
            commands.append(.peer(peer))
        }

        return commands
    }

}
