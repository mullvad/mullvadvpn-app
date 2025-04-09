//
//  ProtocolObfuscationStub.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadSettings
@testable import MullvadTypes
@testable import PacketTunnelCore

struct ProtocolObfuscationStub: ProtocolObfuscation {
    var remotePort: UInt16 { 42 }

    func obfuscate(
        _ endpoint: MullvadEndpoint,
        settings: LatestTunnelSettings,
        retryAttempts: UInt
    ) -> ProtocolObfuscationResult {
        .init(endpoint: endpoint, method: .off)
    }

    var transportLayer: TransportLayer? { .udp }
}
