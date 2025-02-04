//
//  EphemeralPeerKey.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import WireGuardKitTypes

/// The preshared / private key  used by ephemeral peers
public struct EphemeralPeerKey: Equatable {
    public let preSharedKey: PreSharedKey?
    public let ephemeralKey: PrivateKey

    public init(preSharedKey: PreSharedKey? = nil, ephemeralKey: PrivateKey) {
        self.preSharedKey = preSharedKey
        self.ephemeralKey = ephemeralKey
    }
}
