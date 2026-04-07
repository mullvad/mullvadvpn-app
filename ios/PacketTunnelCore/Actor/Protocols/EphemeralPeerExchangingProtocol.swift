//
//  PostQuantumKeyExchangingProtocol.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public protocol EphemeralPeerExchangingProtocol {
    func start() async
    func receivePostQuantumKey(
        _ preSharedKey: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async
    func receiveEphemeralPeerPrivateKey(_: WireGuard.PrivateKey, daitaParameters: DaitaV2Parameters?) async
}
