//
//  TunnelObfuscationStub.swift
//  PacketTunnelCoreTests
//
//  Created by Marco Nikic on 2023-11-21.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadTypes
import Network
@testable import TunnelObfuscation

struct TunnelObfuscationStub: TunnelObfuscation {
    var transportLayer: TransportLayer { .udp }

    let remotePort: UInt16
    init(remoteAddress: IPAddress, tcpPort: UInt16) {
        remotePort = tcpPort
    }

    func start() {}

    func stop() {}

    var localUdpPort: UInt16 { 42 }
}
