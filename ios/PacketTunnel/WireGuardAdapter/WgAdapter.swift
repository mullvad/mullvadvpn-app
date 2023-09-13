//
//  WgAdapter.swift
//  PacketTunnel
//
//  Created by pronebird on 29/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import NetworkExtension
import PacketTunnelCore
import WireGuardKit

struct WgAdapter: TunnelAdapterProtocol {
    let adapter: WireGuardAdapter

    init(packetTunnelProvider: NEPacketTunnelProvider) {
        let logger = Logger(label: "WireGuard")

        adapter = WireGuardAdapter(
            with: packetTunnelProvider,
            shouldHandleReasserting: false,
            logHandler: { logLevel, string in
                logger.log(level: logLevel.loggerLevel, "\(string)")
            }
        )
    }

    func start(configuration: TunnelAdapterConfiguration) async throws {
        try await adapter.start(tunnelConfiguration: configuration.asWgConfig)
    }

    func stop() async throws {
        try await adapter.stop()
    }

    func update(configuration: TunnelAdapterConfiguration) async throws {
        try await adapter.update(tunnelConfiguration: configuration.asWgConfig)
    }
}

extension WgAdapter: TunnelDeviceInfoProtocol {
    var interfaceName: String? {
        return adapter.interfaceName
    }

    func getStats() throws -> WgStats {
        var result: String?

        let dispatchGroup = DispatchGroup()
        dispatchGroup.enter()
        adapter.getRuntimeConfiguration { string in
            result = string
            dispatchGroup.leave()
        }

        guard case .success = dispatchGroup.wait(wallTimeout: .now() + .seconds(1))
        else { throw StatsError.timeout }
        guard let result else { throw StatsError.nilValue }
        guard let newStats = WgStats(from: result) else { throw StatsError.parse }

        return newStats
    }

    enum StatsError: LocalizedError {
        case timeout, nilValue, parse

        var errorDescription: String? {
            switch self {
            case .timeout:
                return "adapter.getRuntimeConfiguration timeout."
            case .nilValue:
                return "Received nil string for stats."
            case .parse:
                return "Couldn't parse stats."
            }
        }
    }
}

private extension TunnelAdapterConfiguration {
    var asWgConfig: TunnelConfiguration {
        var interfaceConfig = InterfaceConfiguration(privateKey: privateKey)
        interfaceConfig.addresses = interfaceAddresses
        interfaceConfig.dns = dns.map { DNSServer(address: $0) }
        interfaceConfig.listenPort = 0

        var peers: [PeerConfiguration] = []
        if let peer {
            var peerConfig = PeerConfiguration(publicKey: peer.publicKey)
            peerConfig.endpoint = peer.endpoint.wgEndpoint
            peerConfig.allowedIPs = [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!,
            ]
            peers.append(peerConfig)
        }

        return TunnelConfiguration(
            name: nil,
            interface: interfaceConfig,
            peers: peers
        )
    }
}

private extension AnyIPEndpoint {
    var wgEndpoint: Endpoint {
        switch self {
        case let .ipv4(endpoint):
            return Endpoint(host: .ipv4(endpoint.ip), port: .init(integerLiteral: endpoint.port))
        case let .ipv6(endpoint):
            return Endpoint(host: .ipv6(endpoint.ip), port: .init(integerLiteral: endpoint.port))
        }
    }
}

private extension WgStats {
    init?(from string: String) {
        var bytesReceived: UInt64?
        var bytesSent: UInt64?

        string.enumerateLines { line, stop in
            if bytesReceived == nil, let value = parseValue("rx_bytes=", in: line) {
                bytesReceived = value
            } else if bytesSent == nil, let value = parseValue("tx_bytes=", in: line) {
                bytesSent = value
            }

            if bytesReceived != nil, bytesSent != nil {
                stop = true
            }
        }

        guard let bytesReceived, let bytesSent else {
            return nil
        }

        self.init(bytesReceived: bytesReceived, bytesSent: bytesSent)
    }
}

@inline(__always) private func parseValue(_ prefixKey: String, in line: String) -> UInt64? {
    guard line.hasPrefix(prefixKey) else { return nil }

    let value = line.dropFirst(prefixKey.count)

    return UInt64(value)
}
