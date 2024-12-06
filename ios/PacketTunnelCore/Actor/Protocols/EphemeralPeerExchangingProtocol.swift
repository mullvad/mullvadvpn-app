//
//  PostQuantumKeyExchangingProtocol.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import WireGuardKitTypes

public protocol EphemeralPeerExchangingProtocol {
    func start() async
    func receivePostQuantumKey(_ preSharedKey: PreSharedKey, ephemeralKey: PrivateKey) async
    func receiveEphemeralPeerPrivateKey(_: PrivateKey) async
}
