//
//  EphemeralPeerKey.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

/// The preshared / private key  used by ephemeral peers
public struct EphemeralPeerKey: Equatable {
    public let preSharedKey: WireGuard.PreSharedKey?
    public let ephemeralKey: WireGuard.PrivateKey

    public init(preSharedKey: WireGuard.PreSharedKey? = nil, ephemeralKey: WireGuard.PrivateKey) {
        self.preSharedKey = preSharedKey
        self.ephemeralKey = ephemeralKey
    }
}
