//
//  ProtocolObfuscator.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes

public protocol ProtocolObfuscation {
    func obfuscate(_ endpoint: MullvadEndpoint, settings: Settings, retryAttempts: UInt) -> MullvadEndpoint
    var transportLayer: TransportLayer? { get }
    var remotePort: UInt16 { get }
}

public class ProtocolObfuscator<Obfuscator: TunnelObfuscation>: ProtocolObfuscation {
    var tunnelObfuscator: TunnelObfuscation?

    public init() {}

    /// Obfuscates a Mullvad endpoint based on a number of retry attempts.
    ///
    /// This retry logic used is explained at the following link
    /// https://github.com/mullvad/mullvadvpn-app/blob/main/docs/relay-selector.md#default-constraints-for-tunnel-endpoints
    /// - Parameters:
    ///   - endpoint: The endpoint to obfuscate.
    ///   - settings: Whether obfuscation should be used or not.
    ///   - retryAttempts: The number of times a connection was attempted to `endpoint`
    /// - Returns: `endpoint` if obfuscation is disabled, or an obfuscated endpoint otherwise.
    public var transportLayer: TransportLayer? {
        return tunnelObfuscator?.transportLayer
    }

    private(set) public var remotePort: UInt16 = 0

    public func obfuscate(_ endpoint: MullvadEndpoint, settings: Settings, retryAttempts: UInt = 0) -> MullvadEndpoint {
        var obfuscatedEndpoint = endpoint
        remotePort = endpoint.ipv4Relay.port
        let shouldObfuscate = switch settings.obfuscation.state {
        case .automatic:
            retryAttempts % 4 == 2 || retryAttempts % 4 == 3
        case .on, .udpTcp, .shadowsocks:
            true
        case .off:
            false
        }

        guard shouldObfuscate else {
            tunnelObfuscator = nil
            return endpoint
        }
        var tcpPort = settings.obfuscation.port
        if tcpPort == .automatic {
            tcpPort = retryAttempts % 2 == 0 ? .port80 : .port5001
        }
        let obfuscator = Obfuscator(
            remoteAddress: obfuscatedEndpoint.ipv4Relay.ip,
            tcpPort: tcpPort.portValue
        )
        remotePort = tcpPort.portValue
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
