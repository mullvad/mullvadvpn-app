//
//  GotaAdapter.swift
//  PacketTunnel
//
//  Created by Emils on 31/10/2025.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes
import Network
import NetworkExtension
import PacketTunnelCore
import WireGuardKitC
@preconcurrency import WireGuardKitTypes

// TODO:
// - NAT64
// - DAITA

public final class GotaAdapter: TunnelAdapterProtocol, TunnelDeviceInfoProtocol, Sendable {

    public enum Error: Swift.Error {
        case noPeer
        case noFileDescriptor
        case notImplemented
    }

    let provider: PacketTunnelProvider
    private nonisolated(unsafe) let gotaTun = GotaTun()
    nonisolated(unsafe) var bytesReceived: UInt64 = 0
    nonisolated(unsafe) var bytesSent: UInt64 = 0

    init(provider: PacketTunnelProvider) {
        self.provider = provider
    }
    /// Returns tunnel interface name (i.e utun0) if available.
    public let interfaceName: String? = "utun0"

    /// Returns tunnel statistics.
    public func getStats() throws -> WgStats {
        bytesReceived += 64
        bytesSent += 64
        return WgStats(bytesReceived: bytesReceived, bytesSent: bytesSent)
    }

    /// Tunnel device file descriptor.
    private var tunnelFileDescriptor: Int32? {
        var ctlInfo = ctl_info()
        withUnsafeMutablePointer(to: &ctlInfo.ctl_name) {
            $0.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: $0.pointee)) {
                _ = strcpy($0, "com.apple.net.utun_control")
            }
        }
        for fd: Int32 in 0...1024 {
            var addr = sockaddr_ctl()
            var ret: Int32 = -1
            var len = socklen_t(MemoryLayout.size(ofValue: addr))
            withUnsafeMutablePointer(to: &addr) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    ret = getpeername(fd, $0, &len)
                }
            }
            if ret != 0 || addr.sc_family != AF_SYSTEM {
                continue
            }
            if ctlInfo.ctl_id == 0 {
                ret = ioctl(fd, CTLIOCGINFO, &ctlInfo)
                if ret != 0 {
                    continue
                }
            }
            if addr.sc_id == ctlInfo.ctl_id {
                return fd
            }
        }
        return nil
    }

    public func start(
        configuration: PacketTunnelCore.TunnelAdapterConfiguration, daita: WireGuardKitTypes.DaitaConfiguration?
    ) async throws {
        try await self.provider.setTunnelNetworkSettings(generateNetworkSettings(for: configuration))

        guard let tunnelFileDescriptor = self.tunnelFileDescriptor else {
            throw Error.noFileDescriptor
        }

        try gotaTun.start(tunnelFileDescriptor: tunnelFileDescriptor, configuration: try gotaConfig(for: configuration))
    }

    private func gotaConfig(for configuration: PacketTunnelCore.TunnelAdapterConfiguration) throws -> GotaTunConfig {
        guard let peer = configuration.peer else {
            throw Error.noPeer
        }

        let config = GotaTunConfig()
        config.addExit(
            privateKey: configuration.privateKey.rawValue,
            preSharedKey: peer.preSharedKey?.rawValue,
            publicKey: peer.publicKey.rawValue,
            endpoint: peer.endpoint.description)

        return config
    }

    public func startMultihop(
        entryConfiguration: PacketTunnelCore.TunnelAdapterConfiguration?,
        exitConfiguration: PacketTunnelCore.TunnelAdapterConfiguration, daita: WireGuardKitTypes.DaitaConfiguration?
    ) async throws {
        try await start(configuration: exitConfiguration, daita: nil)
    }

    public func stop() async throws {
        gotaTun.stop()
    }

    private func generateNetworkSettings(for config: TunnelAdapterConfiguration) -> NEPacketTunnelNetworkSettings {
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")

        let dnsServerStrings = config.dns.map { "\($0)" }
        let dnsSettings = NEDNSSettings(servers: dnsServerStrings)
        dnsSettings.matchDomains = [""]

        networkSettings.dnsSettings = dnsSettings
        networkSettings.mtu = NSNumber(value: 1280)

        let v4Addresses: [NEIPv4Route] = config.interfaceAddresses.compactMap({ address in
            guard address.address is IPv4Address else {
                return nil
            }

            return NEIPv4Route(destinationAddress: "\(address.address)", subnetMask: "\(address.subnetMask())")
        })

        networkSettings.ipv4Settings = NEIPv4Settings(
            addresses: v4Addresses.map { $0.destinationAddress },
            subnetMasks: v4Addresses.map { $0.destinationSubnetMask })
        networkSettings.ipv4Settings?.includedRoutes = [NEIPv4Route.default()]

        let v6Addresses: [NEIPv6Route] = config.interfaceAddresses.compactMap({ address in
            guard address.address is IPv6Address else {
                return nil
            }

            return NEIPv6Route(
                destinationAddress: "\(address.address)",
                networkPrefixLength: NSNumber(value: min(120, address.networkPrefixLength)))
        })

        networkSettings.ipv6Settings = NEIPv6Settings(
            addresses: v6Addresses.map { $0.destinationAddress },
            networkPrefixLengths: v6Addresses.map { $0.destinationNetworkPrefixLength })
        networkSettings.ipv6Settings?.includedRoutes = [NEIPv6Route.default()]

        return networkSettings

    }
}
