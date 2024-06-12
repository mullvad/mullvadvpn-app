//
//  NEPacketTunnelProvider+Extensions.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2024-06-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

public protocol TunnelProvider: AnyObject {
    func createTCPConnectionThroughTunnel(
        to remoteEndpoint: NWEndpoint,
        enableTLS: Bool,
        tlsParameters TLSParameters: NWTLSParameters?,
        delegate: Any?
    ) -> NWTCPConnection
}

extension NEPacketTunnelProvider: TunnelProvider {}
