//
//  ProtocolObfuscator.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2023-11-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

public struct ProtocolObfuscationResult {
    let endpoint: MullvadEndpoint
    let method: WireGuardObfuscationState
}

public protocol ProtocolObfuscation {
    func obfuscate(
        _ endpoint: MullvadEndpoint,
        settings: LatestTunnelSettings,
        retryAttempts: UInt,
        relayFeatures: REST.ServerRelay.Features?
    ) -> ProtocolObfuscationResult
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
        retryAttempts: UInt = 0,
        relayFeatures: REST.ServerRelay.Features?
    ) -> ProtocolObfuscationResult {
        let obfuscationMethod = ObfuscationMethodSelector.obfuscationMethodBy(
            connectionAttemptCount: retryAttempts,
            tunnelSettings: settings
        )

        remotePort = endpoint.ipv4Relay.port

        #if DEBUG
        let obfuscationProtocol: TunnelObfuscationProtocol? = switch obfuscationMethod {
        case .udpOverTcp:
            .udpOverTcp
        case .shadowsocks:
            .shadowsocks
        case .quic:
            if let relayFeatures = relayFeatures?.quic {
                .quic(hostname: relayFeatures.domain, token: relayFeatures.token)
            } else {
                nil
            }
        default:
            // This is fine, since ObfuscationMethodSelector.obfuscationMethodBy` above should never
            // return .automatic.
            nil
        }

        guard let obfuscationProtocol else {
            tunnelObfuscator = nil
            return .init(endpoint: endpoint, method: .off)
        }

        let obfuscator = Obfuscator(
            remoteAddress: endpoint.ipv4Relay.ip,
            tcpPort: remotePort,
            obfuscationProtocol: obfuscationProtocol
        )
        #else
        // At this point, the only possible obfuscation methods should be either `.udpOverTcp` or `.shadowsocks`
        let obfuscator = Obfuscator(
            remoteAddress: endpoint.ipv4Relay.ip,
            tcpPort: remotePort,
            obfuscationProtocol: obfuscationMethod == .shadowsocks ? .shadowsocks : .udpOverTcp
        )
        #endif

        obfuscator.start()
        tunnelObfuscator = obfuscator

        return .init(
            endpoint: MullvadEndpoint(
                ipv4Relay: IPv4Endpoint(ip: .loopback, port: obfuscator.localUdpPort),
                ipv4Gateway: endpoint.ipv4Gateway,
                ipv6Gateway: endpoint.ipv6Gateway,
                publicKey: endpoint.publicKey
            ),
            method: obfuscationMethod
        )
    }
}
