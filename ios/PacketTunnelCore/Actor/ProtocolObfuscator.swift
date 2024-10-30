//
//  ProtocolObfuscator.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

public protocol ProtocolObfuscation {
    func obfuscate(_ endpoint: MullvadEndpoint, settings: LatestTunnelSettings, retryAttempts: UInt) -> MullvadEndpoint
    var transportLayer: TransportLayer? { get }
    var remotePort: UInt16 { get }
}

public class ProtocolObfuscator<Obfuscator: TunnelObfuscation>: ProtocolObfuscation {
    var tunnelObfuscator: TunnelObfuscation?

    public init() {}

    /// Obfuscates a Mullvad endpoint.
    ///
    /// - Parameters:
    ///   - endpoint: The endpoint to obfuscate.
    /// - Returns: `endpoint` if obfuscation is disabled, or an obfuscated endpoint otherwise.
    public var transportLayer: TransportLayer? {
        return tunnelObfuscator?.transportLayer
    }

    private(set) public var remotePort: UInt16 = 0

    public func obfuscate(
        _ endpoint: MullvadEndpoint,
        settings: LatestTunnelSettings,
        retryAttempts: UInt = 0
    ) -> MullvadEndpoint {
        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: retryAttempts,
            tunnelSettings: settings
        )

        guard obfuscationMethod != .off else {
            tunnelObfuscator = nil
            return endpoint
        }

        remotePort = endpoint.ipv4Relay.port

        let obfuscator = Obfuscator(
            remoteAddress: endpoint.ipv4Relay.ip,
            tcpPort: remotePort
        )

        obfuscator.start()
        tunnelObfuscator = obfuscator

        return MullvadEndpoint(
            ipv4Relay: IPv4Endpoint(ip: .loopback, port: obfuscator.localUdpPort),
            ipv4Gateway: endpoint.ipv4Gateway,
            ipv6Gateway: endpoint.ipv6Gateway,
            publicKey: endpoint.publicKey
        )
    }
}
