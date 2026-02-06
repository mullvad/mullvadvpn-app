//
//  ProtocolObfuscator.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

public struct ProtocolObfuscationResult {
    public let endpoint: SelectedEndpoint
}

public protocol ProtocolObfuscation {
    func obfuscate(_ endpoint: SelectedEndpoint) -> ProtocolObfuscationResult
    var transportLayer: TransportLayer? { get }
    var remotePort: UInt16 { get }
}

public class ProtocolObfuscator<Obfuscator: TunnelObfuscation>: ProtocolObfuscation {
    var tunnelObfuscator: TunnelObfuscation?

    public init() {}

    public var transportLayer: TransportLayer? {
        return tunnelObfuscator?.transportLayer
    }

    private(set) public var remotePort: UInt16 = 0

    /// Obfuscates a selected endpoint if obfuscation is enabled.
    ///
    /// - Parameters:
    ///   - endpoint: The endpoint to obfuscate. Contains socket address and obfuscation method.
    /// - Returns: The endpoint (possibly modified) with obfuscation applied.
    ///
    /// Note: Obfuscation currently only supports IPv4. If the endpoint uses IPv6,
    /// obfuscation is skipped and the endpoint is returned as-is with obfuscation disabled.
    public func obfuscate(_ endpoint: SelectedEndpoint) -> ProtocolObfuscationResult {
        remotePort = endpoint.socketAddress.port

        // Extract obfuscation protocol from the bundled obfuscation method
        let obfuscationProtocol: TunnelObfuscationProtocol? =
            switch endpoint.obfuscation {
            case .off:
                nil
            case .udpOverTcp:
                .udpOverTcp
            case .shadowsocks:
                .shadowsocks
            case let .quic(hostname, token):
                .quic(hostname: hostname, token: token)
            }

        // If obfuscation is disabled, return endpoint as-is
        guard let obfuscationProtocol else {
            tunnelObfuscator = nil
            return .init(endpoint: endpoint)
        }

        let obfuscator = Obfuscator(
            remoteAddress: endpoint.socketAddress.ip,
            tcpPort: remotePort,
            obfuscationProtocol: obfuscationProtocol
        )

        obfuscator.start()
        tunnelObfuscator = obfuscator

        let localAddress: AnyIPEndpoint =
            switch endpoint.socketAddress {
            case .ipv4:
                .ipv4(IPv4Endpoint(ip: .loopback, port: obfuscator.localUdpPort))
            case .ipv6:
                .ipv6(IPv6Endpoint(ip: .loopback, port: obfuscator.localUdpPort))
            }

        // Return endpoint with loopback address pointing to local obfuscation proxy
        let obfuscatedEndpoint = SelectedEndpoint(
            socketAddress: localAddress,
            ipv4Gateway: endpoint.ipv4Gateway,
            ipv6Gateway: endpoint.ipv6Gateway,
            publicKey: endpoint.publicKey,
            obfuscation: endpoint.obfuscation
        )

        return .init(endpoint: obfuscatedEndpoint)
    }
}
