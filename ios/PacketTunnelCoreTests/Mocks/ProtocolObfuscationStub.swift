//
//  ProtocolObfuscationStub.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadTypes
@testable import PacketTunnelCore

struct ProtocolObfuscationStub: ProtocolObfuscation {
    func obfuscate(_ endpoint: MullvadEndpoint, settings: Settings, retryAttempts: UInt) -> MullvadEndpoint {
        endpoint
    }

    var transportLayer: TransportLayer? { .udp }
}
