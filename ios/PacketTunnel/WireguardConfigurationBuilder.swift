//
//  WireguardConfigurationBuilder.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network

class WireguardConfigurationBuilder {
    private var privateKey: WireguardPrivateKey?
    private var listenPort: UInt16?
    private var replacePeers: Bool?
    private var peers = [(AnyIPEndpoint, Data)]()
    private var replaceAllowedIPs: Bool?
    private var allowedIPs = [IPAddressRange]()

    @discardableResult func privateKey(_ privateKey: WireguardPrivateKey) -> Self {
        self.privateKey = privateKey
        return self
    }

    @discardableResult func listenPort(_ port: UInt16) -> Self {
        self.listenPort = port
        return self
    }

    @discardableResult func peer(_ endpoint: AnyIPEndpoint, publicKey: Data) -> Self {
        self.peers.append((endpoint, publicKey))
        return self
    }

    @discardableResult func replacePeers(_ value: Bool) -> Self {
        self.replacePeers = value
        return self
    }

    @discardableResult func replaceAllowedIPs(_ value: Bool) -> Self {
        self.replaceAllowedIPs = value
        return self
    }

    @discardableResult func allowedIp(_ addressRange: IPAddressRange) -> Self {
        self.allowedIPs.append(addressRange)
        return self
    }

    @discardableResult func build() -> String {
        var config = [String]()

        if let privateKey = privateKey {
            let keyString = privateKey.rawRepresentation.hexEncodedString()

            config.append("private_key=\(keyString)")
        }

        if let listenPort = listenPort {
            config.append("listen_port=\(listenPort)")
        }

        if let replacePeers = replacePeers, replacePeers {
            config.append("replace_peers=\(replacePeers)")
        }

        for (endpoint, publicKey) in peers {
            let keyString = publicKey.hexEncodedString()

            config.append("public_key=\(keyString)")
            config.append("endpoint=\(endpoint.wireguardStringRepresentation)")
        }

        if let replaceAllowedIPs = replaceAllowedIPs, replaceAllowedIPs {
            config.append("replace_allowed_ips=\(replaceAllowedIPs)")
        }

        for allowedIP in allowedIPs {
            config.append("allowed_ip=\(allowedIP)")
        }

        return config.joined(separator: "\n")
    }

}
