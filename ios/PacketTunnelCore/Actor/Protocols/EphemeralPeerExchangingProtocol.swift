//
//  PostQuantumKeyExchangingProtocol.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import WireGuardKitTypes

public protocol EphemeralPeerExchangingProtocol {
    func start()
    func receivePostQuantumKey(_ preSharedKey: PreSharedKey, ephemeralKey: PrivateKey)
    func receiveEphemeralPeerPrivateKey(_: PrivateKey)
}
