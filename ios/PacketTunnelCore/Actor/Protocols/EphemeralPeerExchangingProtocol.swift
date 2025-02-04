//
//  PostQuantumKeyExchangingProtocol.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import WireGuardKitTypes

public protocol EphemeralPeerExchangingProtocol {
    func start() async
    func receivePostQuantumKey(
        _ preSharedKey: PreSharedKey,
        ephemeralKey: PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async
    func receiveEphemeralPeerPrivateKey(_: PrivateKey, daitaParameters: DaitaV2Parameters?) async
}
