//
//  PacketTunnelSettingsGenerator.swift
//  PacketTunnel
//
//  Created by pronebird on 13/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network
import NetworkExtension
import os

struct PacketTunnelSettingsGenerator {
    let mullvadEndpoint: MullvadEndpoint
    let tunnelConfiguration: TunnelConfiguration

    func networkSettings() -> NEPacketTunnelNetworkSettings {
        let tunnelRemoteAddress = "\(mullvadEndpoint.ipv4Relay.ip)"
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: tunnelRemoteAddress)

        networkSettings.mtu = 1280
        networkSettings.dnsSettings = dnsSettings()
        networkSettings.ipv4Settings = ipv4Settings()
        networkSettings.ipv6Settings = ipv6Settings()

        return networkSettings
    }

    private func dnsSettings() -> NEDNSSettings {
        let serverAddresses = [mullvadEndpoint.ipv4Gateway, mullvadEndpoint.ipv6Gateway]
            .map { String(reflecting: $0) }

        let dnsSettings = NEDNSSettings(servers: serverAddresses)

        // All DNS queries must first go through the tunnel's DNS
        dnsSettings.matchDomains = [""]

        return dnsSettings
    }

    private func ipv4Settings() -> NEIPv4Settings {
        let interfaceAddresses = tunnelConfiguration.interface.addresses
        let ipv4AddressRanges = interfaceAddresses.filter { $0.address is IPv4Address }

        let ipv4Settings = NEIPv4Settings(
            addresses: ipv4AddressRanges.map { "\($0.address)" },
            subnetMasks: ipv4AddressRanges.map { self.ipv4SubnetMaskString(of: $0) })

        ipv4Settings.includedRoutes = [
            NEIPv4Route.default() // 0.0.0.0/0
        ]

        let relayAddressRange = IPAddressRange(address: mullvadEndpoint.ipv4Relay.ip, networkPrefixLength: 32)

        ipv4Settings.excludedRoutes = [
            NEIPv4Route(
                destinationAddress: "\(relayAddressRange.address)",
                subnetMask: ipv4SubnetMaskString(of: relayAddressRange))
        ]

        return ipv4Settings
    }

    private func ipv6Settings() -> NEIPv6Settings {
        let interfaceAddresses = tunnelConfiguration.interface.addresses
        let ipv6AddressRanges = interfaceAddresses.filter { $0.address is IPv6Address }

        let ipv6Settings = NEIPv6Settings(
            addresses: ipv6AddressRanges.map { "\($0.address)" },
            networkPrefixLengths: ipv6AddressRanges.map { NSNumber(value: $0.networkPrefixLength) }
        )

        ipv6Settings.includedRoutes = [
            NEIPv6Route.default() // ::0
        ]

        if let ipv6Relay = mullvadEndpoint.ipv6Relay {
            ipv6Settings.excludedRoutes = [
                NEIPv6Route(destinationAddress: "\(ipv6Relay.ip)", networkPrefixLength: 128)
            ]
        }

        return ipv6Settings
    }

    private func ipv4SubnetMaskString(of addressRange: IPAddressRange) -> String {
        let length: UInt8 = addressRange.networkPrefixLength
        assert(length <= 32)
        var octets: [UInt8] = [0, 0, 0, 0]
        let subnetMask: UInt32 = length > 0 ? ~UInt32(0) << (32 - length) : UInt32(0)
        octets[0] = UInt8(truncatingIfNeeded: subnetMask >> 24)
        octets[1] = UInt8(truncatingIfNeeded: subnetMask >> 16)
        octets[2] = UInt8(truncatingIfNeeded: subnetMask >> 8)
        octets[3] = UInt8(truncatingIfNeeded: subnetMask)
        return octets.map { String($0) }.joined(separator: ".")
    }
}
