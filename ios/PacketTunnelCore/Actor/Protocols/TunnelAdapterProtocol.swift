//
//  TunnelAdapterProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import NetworkExtension
@preconcurrency import WireGuardKitTypes

/// Protocol describing interface for any kind of adapter implementing a VPN tunnel.
public protocol TunnelAdapterProtocol: Sendable {
    /// Start tunnel adapter or update active configuration.
    /// # Important
    /// Call `apply` at least once with a valid configuration for the tunnel to receive any user traffic.
    func start(configuration: TunnelAdapterConfiguration, daita: DaitaConfiguration?) async throws

    /// Start tunnel adapter or update active configuration.
    /// # Important
    /// Call `apply` at least once with a valid configuration for the tunnel to receive any user traffic.
    func startMultihop(
        entryConfiguration: TunnelAdapterConfiguration?,
        exitConfiguration: TunnelAdapterConfiguration,
        daita: DaitaConfiguration?
    ) async throws

    /// Stop tunnel adapter with the given configuration.
    func stop() async throws

    /// Applies tunnel interface settings, without configuring WireGuard.
    /// # Important
    ///  Applying tunnel interface settings should be done at least once before starting the tunnel via `start` or `startMultihop`.
    ///  Without applying tunnel interface settings, iOS will not route traffic through the tunnel, so it is important to call this function as soon as possible.
    func apply(settings: TunnelInterfaceSettings) async throws

}

/// Struct describing tunnel adapter configuration.
public struct TunnelAdapterConfiguration {
    public var privateKey: PrivateKey
    public var interfaceAddresses: [IPAddressRange]
    public var dns: [IPAddress]
    public var peer: TunnelPeer?
    public var allowedIPs: [IPAddressRange]
    public var pingableGateway: IPv4Address

    func asTunnelSettings() -> TunnelInterfaceSettings {
        return TunnelInterfaceSettings(
            interfaceAddresses: self.interfaceAddresses,
            dns: self.dns,
        )
    }
}

/// Encapsulates all data needed to call PacketTunnelProvider.SetTunnelNetworkSettings()
public struct TunnelInterfaceSettings: Equatable, Sendable {
    public var interfaceAddresses: [IPAddressRange]
    public var dns: [IPAddress]

    public func asTunnelSettings() -> NEPacketTunnelNetworkSettings {
        let networkSettings = NEPacketTunnelNetworkSettings(
            tunnelRemoteAddress: "\(IPv4Address.loopback)")

        let dnsSettings = NEDNSSettings(servers: self.dns.map({ "\($0)" }))
        dnsSettings.matchDomains = [""]  // All DNS queries must first go through the tunnel's DNS
        networkSettings.dnsSettings = dnsSettings
        networkSettings.mtu = NSNumber(value: 1280)

        networkSettings.ipv4Settings = v4Configuration()
        networkSettings.ipv6Settings = v6Configuration()

        return networkSettings
    }

    func v4Configuration() -> NEIPv4Settings {
        var addresses = [String]()
        var subnetMasks = [String]()
        var routes = [NEIPv4Route]()
        routes.append(NEIPv4Route.default())

        for range in self.interfaceAddresses where range.address is IPv4Address {
            addresses.append("\(range.address)")
            subnetMasks.append("\(range.subnetMask())")

            let route = NEIPv4Route(
                destinationAddress: "\(range.maskedAddress())",
                subnetMask: "\(range.subnetMask())"
            )
            route.gatewayAddress = "\(range.address)"
            routes.append(route)
        }

        let settings = NEIPv4Settings(addresses: addresses, subnetMasks: subnetMasks)
        settings.includedRoutes = routes
        return settings
    }

    func v6Configuration() -> NEIPv6Settings {
        var addresses = [String]()
        var prefixLengths = [NSNumber]()
        var routes = [NEIPv6Route]()

        for range in self.interfaceAddresses where range.address is IPv6Address {
            addresses.append("\(range.address)")
            prefixLengths.append(NSNumber(value: min(120, range.networkPrefixLength)))

            let route = NEIPv6Route(
                destinationAddress: "\(range.maskedAddress())",
            networkPrefixLength: NSNumber(value: range.networkPrefixLength))
            route.gatewayAddress = "\(range.address)"
            routes.append(route)
        }

        let settings = NEIPv6Settings(
            addresses: addresses,
            networkPrefixLengths: prefixLengths
        )
        settings.includedRoutes = routes
        return settings
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        if lhs.interfaceAddresses != rhs.interfaceAddresses {
            return false
        }

        let lhsSet = Set(lhs.dns.map { "\($0)" })
        let rhsSet = Set(rhs.dns.map { "\($0)" })
        if lhsSet != rhsSet {
            return false
        }

        return true
    }

}

/// Struct describing a single peer.
public struct TunnelPeer {
    public var endpoint: AnyIPEndpoint
    public var publicKey: PublicKey
    public var preSharedKey: PreSharedKey?
}

extension TunnelAdapterConfiguration: CustomDebugStringConvertible {
    public var debugDescription: String {
        "interfaceAddresses: \(interfaceAddresses) peerEndpoint: \(peer?.endpoint.description ?? "") allowedIPs: \(allowedIPs)"
    }
}
