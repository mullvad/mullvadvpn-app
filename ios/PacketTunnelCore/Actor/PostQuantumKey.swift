//
//  PostQuantumKey.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import WireGuardKitTypes

public struct PostQuantumKey: Equatable {
    public let preSharedKey: PreSharedKey
    public let ephemeralKey: PrivateKey

    public init(preSharedKey: PreSharedKey, ephemeralKey: PrivateKey) {
        self.preSharedKey = preSharedKey
        self.ephemeralKey = ephemeralKey
    }
}
