//
//  PostQuantumKeyExchangeActor.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-04-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadPostQuantum
import NetworkExtension

typealias InTunnelTCPConnectionCreator = (NWHostEndpoint) -> NWTCPConnection

actor PostQuantumKeyExchangeActor {
    let createNetworkConnection: InTunnelTCPConnectionCreator
    unowned let packetTunnel: PacketTunnelProvider
    private var quantumKeyNegotiatior: PostQuantumKeyNegotiatior!
    private var inTunnelTCPConnection: NWTCPConnection!
    private var tcpConnectionObserver: NSKeyValueObservation!

    init(packetTunnel: PacketTunnelProvider, createNetworkConnection: @escaping InTunnelTCPConnectionCreator) {
        self.packetTunnel = packetTunnel
        self.createNetworkConnection = createNetworkConnection
    }
}
