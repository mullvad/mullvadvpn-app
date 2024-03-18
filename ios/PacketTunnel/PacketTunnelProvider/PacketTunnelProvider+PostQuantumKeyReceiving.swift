//
//  PacketTunnelProvider+PostQuantumKeyReceiving.swift
//  PacketTunnel
//
//  Created by Andrew Bulhak on 2024-03-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import WireGuardKitTypes

extension PacketTunnelProvider: PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ key: PreSharedKey) {
        // TODO: send the key to the actor
    }
}
