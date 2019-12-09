//
//  WireguardCommand.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

struct WireguardPeer {
    let endpoint: AnyIPEndpoint
    let publicKey: Data
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
            let endpointString = peer.endpoint.wireguardStringRepresentation

            return ["public_key=\(keyString)", "endpoint=\(endpointString)"]
                .joined(separator: "\n")

        case .replaceAllowedIPs:
            return "replace_allowed_ips=true"

        case .allowedIP(let ipAddressRange):
            return "allowed_ip=\(ipAddressRange)"
        }
    }

}

extension Array where Element == WireguardCommand {
    func toWireguardConfig() -> String {
        map { $0.toRawWireguardCommand() }
            .joined(separator: "\n")
    }
}
