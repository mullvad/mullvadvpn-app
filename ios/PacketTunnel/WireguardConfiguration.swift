//
//  WireguardConfiguration.swift
//  PacketTunnel
//
//  Created by pronebird on 17/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import os

/// A struct describing a basic WireGuard configuration
struct WireguardConfiguration {
    var privateKey: WireguardPrivateKey
    var peers: [WireguardPeer]
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
            commands.append(.allowedIP(peer.anyAllowedIP))
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

        if oldPeers != newPeers {
            let oldPublicKeys = Set(oldPeers.map { $0.publicKey })
            let newPublicKeys = Set(newPeers.map { $0.publicKey })

            // Avoid using `replace_peers` when updating the existing peers.
            if oldPublicKeys != newPublicKeys {
                commands.append(.replacePeers)
            }

            newPeers.forEach { (peer) in
                commands.append(.peer(peer))
                commands.append(.allowedIP(peer.anyAllowedIP))
            }
        }

        return commands
    }

    func withReresolvedPeers(maxRetryOnFailure: Int = 0) -> AnyPublisher<WireguardConfiguration, Error> {
        self.peers
            .publisher
            .setFailureType(to: Error.self)
            .flatMap {
                $0.withReresolvedEndpoint()
                    .publisher
                    .retry(maxRetryOnFailure)

        }
        .collect()
        .map({ (peers) in
            WireguardConfiguration(
                privateKey: self.privateKey,
                peers: peers
            )
        })
            .eraseToAnyPublisher()
    }

}
