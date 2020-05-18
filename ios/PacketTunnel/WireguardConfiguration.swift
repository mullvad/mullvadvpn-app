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

    /// Returns a baseline configuration for WireGuard
    func baseline() -> [WireguardCommand] {
        var commands: [WireguardCommand] = [
            .privateKey(privateKey),
            .listenPort(0),
            .replacePeers
        ]

        peers.forEach { (peer) in
            commands.append(.peer(peer))
        }

        allowedIPs.forEach { (ipAddressRange) in
            commands.append(.allowedIP(ipAddressRange))
        }

        return commands
    }

    /// Returns a WireGuard configuration for transition to the given configuration
    func transition(to newConfig: WireguardConfiguration) -> [WireguardCommand] {
        var commands = [WireguardCommand]()

        if self.privateKey != newConfig.privateKey {
            commands.append(.privateKey(newConfig.privateKey))
        }

        let oldPeers = Set(self.peers)
        let newPeers = Set(newConfig.peers)
        let oldPublicKeys = Set(oldPeers.map { $0.publicKey })
        let newPublicKeys = Set(newPeers.map { $0.publicKey })
        let shouldReplacePeers = oldPublicKeys != newPublicKeys

        if oldPeers != newPeers {
            // Avoid using `replace_peers` when updating the existing peers.
            if shouldReplacePeers {
                commands.append(.replacePeers)
            }

            newPeers.forEach { (peer) in
                commands.append(.peer(peer))
            }
        }

        let oldAllowedIPs = Set(self.allowedIPs)
        let newAllowedIPs = Set(newConfig.allowedIPs)

        // It looks like the `allowed_ip` table is being flushed when `replace_peers=true` is passed
        if oldAllowedIPs != newAllowedIPs || shouldReplacePeers {
            commands.append(.replaceAllowedIPs)

            newAllowedIPs.forEach { (allowedIP) in
                commands.append(.allowedIP(allowedIP))
            }
        }

        return commands
    }

}
