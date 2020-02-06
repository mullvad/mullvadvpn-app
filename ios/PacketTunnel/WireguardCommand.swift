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

    func withReresolvedEndpoint() -> Result<WireguardPeer, Error> {
        self.endpoint.withReresolvedIP()
            .map { WireguardPeer(endpoint: $0, publicKey: self.publicKey) }
    }

}

extension WireguardPeer {

    /// Returns a 0.0.0.0/0 for IPv4 and ::0/0 for IPv6
    var anyAllowedIP: IPAddressRange {
        switch endpoint {
        case .ipv4:
            return IPAddressRange(address: IPv4Address.any, networkPrefixLength: 0)
        case .ipv6:
            return IPAddressRange(address: IPv6Address.any, networkPrefixLength: 0)
        }
    }
}

enum WireguardCommand {
    case privateKey(WireguardPrivateKey)
    case listenPort(UInt16)
    case replacePeers
    case peer(WireguardPeer)
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

        case .allowedIP(let ipAddressRange):
            return "allowed_ip=\(ipAddressRange)"
        }
    }

}

extension Array where Element == WireguardCommand {
    func toRawWireguardConfigString() -> String {
        map { $0.toRawWireguardCommand() }
            .joined(separator: "\n")
    }
}
