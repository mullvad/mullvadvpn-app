//
//  WireguardCommand.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct WireguardPeer: Hashable {
    let endpoint: AnyIPEndpoint
    let publicKey: Data
}

extension WireguardPeer {

    func withResolvedEndpoint() -> Result<WireguardPeer, Error> {
        return self.endpoint.withResolvedIP().map { (endpoint) -> WireguardPeer in
            return WireguardPeer(endpoint: endpoint, publicKey: self.publicKey)
        }
    }

}

enum WireguardCommand {
    case privateKey(WireguardPrivateKey)
    case listenPort(UInt16)
    case replacePeers
    case peer(WireguardPeer)
    case replaceAllowedIPs
    case allowedIP(IPAddressRange)
}

extension WireguardCommand {

    func toRawWireguardCommand() -> String {
        switch self {
        case .privateKey(let privateKey):
            let keyString = privateKey.rawRepresentation.hexEncodedString()

            return "private_key=\(keyString)"

        case .listenPort(let port):
            return "listen_port=\(port)"

        case .replacePeers:
            return "replace_peers=true"

        case .peer(let peer):
            let keyString = peer.publicKey.hexEncodedString()

            return ["public_key=\(keyString)", "endpoint=\(peer.endpoint)"]
                .joined(separator: "\n")

        case .replaceAllowedIPs:
            return "replace_allowed_ips=true"

        case .allowedIP(let ipAddressRange):
            return "allowed_ip=\(ipAddressRange)"
        }
    }

}

extension Array where Element == WireguardCommand {
    func toRawWireguardConfigString() -> String {
        return map { $0.toRawWireguardCommand() }
            .joined(separator: "\n")
    }
}
