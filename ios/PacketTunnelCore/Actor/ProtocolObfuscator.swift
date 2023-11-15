//
//  ProtocolObfuscator.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import TunnelObfuscation

public protocol ProtocolObfuscation {
    func obfuscate(_ endpoint: MullvadEndpoint, settings: Settings, retryAttempts: UInt) -> MullvadEndpoint
}

public class ProtocolObfuscator<Obfuscator: TunnelObfuscation>: ProtocolObfuscation {
    var tunnelObfuscator: TunnelObfuscation?

    public init() {}

    public func obfuscate(_ endpoint: MullvadEndpoint, settings: Settings, retryAttempts: UInt = 0) -> MullvadEndpoint {
        var obfuscatedEndpoint = endpoint
        let shouldObfuscate = switch settings.obfuscation.state {
        case .automatic:
            retryAttempts % 4 == 2 || retryAttempts % 4 == 3
        case .on:
            true
        case .off:
            false
        }

        guard shouldObfuscate else {
            tunnelObfuscator = nil
            return endpoint
        }
        var tcpPort = settings.obfuscation.port
        if settings.obfuscation.port == .automatic {
            tcpPort = retryAttempts % 2 == 0 ? .port80 : .port5001
        }
        let obfuscator = Obfuscator(
            remoteAddress: obfuscatedEndpoint.ipv4Relay.ip,
            tcpPort: tcpPort.portValue
        )
        obfuscator.start()
        tunnelObfuscator = obfuscator

        let localObfuscatorEndpoint = IPv4Endpoint(ip: .loopback, port: obfuscator.localUdpPort)
        obfuscatedEndpoint = MullvadEndpoint(
            ipv4Relay: localObfuscatorEndpoint,
            ipv4Gateway: obfuscatedEndpoint.ipv4Gateway,
            ipv6Gateway: obfuscatedEndpoint.ipv6Gateway,
            publicKey: obfuscatedEndpoint.publicKey
        )

        return obfuscatedEndpoint
    }
}
