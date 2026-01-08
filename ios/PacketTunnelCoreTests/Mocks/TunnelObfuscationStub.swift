//
//  TunnelObfuscationStub.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import WireGuardKitTypes

@testable import MullvadRustRuntime
@testable import MullvadTypes

struct TunnelObfuscationStub: TunnelObfuscation {
    var transportLayer: TransportLayer { .udp }

    let remotePort: UInt16
    init(
        remoteAddress: IPAddress,
        tcpPort: UInt16,
        obfuscationProtocol: TunnelObfuscationProtocol,
        clientPublicKey: PublicKey
    ) {
        remotePort = tcpPort
    }

    func start() {}

    func stop() {}

    var localUdpPort: UInt16 { 42 }
}
