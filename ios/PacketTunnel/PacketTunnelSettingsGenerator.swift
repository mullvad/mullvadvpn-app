//
//  PacketTunnelSettingsGenerator.swift
//  PacketTunnel
//
//  Created by pronebird on 13/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//  Copyright © 2018-2019 WireGuard LLC. All Rights Reserved.
//

import Foundation
import Network
import NetworkExtension
import WireGuardKit

struct PacketTunnelSettingsGenerator {
    let mullvadEndpoint: MullvadEndpoint
    let tunnelSettings: TunnelSettings

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
            .map { "\($0)" }

        let dnsSettings = NEDNSSettings(servers: serverAddresses)

        // All DNS queries must first go through the tunnel's DNS
        dnsSettings.matchDomains = [""]

        return dnsSettings
    }

    private func ipv4Settings() -> NEIPv4Settings {
        let interfaceAddresses = tunnelSettings.interface.addresses
        let ipv4AddressRanges = interfaceAddresses.filter { $0.address is IPv4Address }

        let ipv4Settings = NEIPv4Settings(
            addresses: ipv4AddressRanges.map { "\($0.address)" },
            subnetMasks: ipv4AddressRanges.map { self.ipv4SubnetMaskString(of: $0) })

        ipv4Settings.includedRoutes = [
            NEIPv4Route.default() // 0.0.0.0/0
        ]

        return ipv4Settings
    }

    private func ipv6Settings() -> NEIPv6Settings {
        let interfaceAddresses = tunnelSettings.interface.addresses
        let ipv6AddressRanges = interfaceAddresses.filter { $0.address is IPv6Address }

        let addresses = ipv6AddressRanges.map { "\($0.address)" }

        // The smallest prefix that will have any effect on iOS is /120
        let networkPrefixLengths = ipv6AddressRanges
            .map { NSNumber(value: min(120, $0.networkPrefixLength)) }

        let ipv6Settings = NEIPv6Settings(
            addresses: addresses,
            networkPrefixLengths: networkPrefixLengths
        )

        ipv6Settings.includedRoutes = [
            NEIPv6Route.default() // ::0
        ]

        return ipv6Settings
    }

    private func ipv4SubnetMaskString(of addressRange: IPAddressRange) -> String {
        let length: UInt8 = addressRange.networkPrefixLength
        assert(length <= 32)
        var octets: [UInt8] = [0, 0, 0, 0]
        let subnetMask: UInt32 = length > 0 ? UInt32.max << (32 - length) : UInt32.zero
        octets[0] = UInt8(truncatingIfNeeded: subnetMask >> 24)
        octets[1] = UInt8(truncatingIfNeeded: subnetMask >> 16)
        octets[2] = UInt8(truncatingIfNeeded: subnetMask >> 8)
        octets[3] = UInt8(truncatingIfNeeded: subnetMask)
        return octets.map { String($0) }.joined(separator: ".")
    }
}
