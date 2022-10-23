//
//  PacketTunnelConfiguration.swift
//  PacketTunnel
//
//  Created by pronebird on 15/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import protocol Network.IPAddress
import RelaySelector
import WireGuardKit

struct PacketTunnelConfiguration {
    var deviceState: DeviceState
    var tunnelSettings: TunnelSettingsV2
    var selectorResult: RelaySelectorResult
}

extension PacketTunnelConfiguration {
    var wgTunnelConfig: TunnelConfiguration {
        let mullvadEndpoint = selectorResult.endpoint
        var peers = [mullvadEndpoint.ipv4RelayEndpoint]
        if let ipv6RelayEndpoint = mullvadEndpoint.ipv6RelayEndpoint {
            peers.append(ipv6RelayEndpoint)
        }

        let peerConfigs = peers.compactMap { endpoint -> PeerConfiguration in
            let pubKey = PublicKey(rawValue: selectorResult.endpoint.publicKey)!
            var peerConfig = PeerConfiguration(publicKey: pubKey)
            peerConfig.endpoint = endpoint
            peerConfig.allowedIPs = [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!,
            ]
            return peerConfig
        }

        var interfaceConfig: InterfaceConfiguration

        switch deviceState {
        case let .loggedIn(_, device):
            interfaceConfig = InterfaceConfiguration(privateKey: device.wgKeyData.privateKey)
            interfaceConfig.addresses = [device.ipv4Address, device.ipv6Address]
            interfaceConfig.dns = dnsServers.map { DNSServer(address: $0) }

        case .loggedOut, .revoked:
            interfaceConfig = InterfaceConfiguration(privateKey: PrivateKey())
        }

        interfaceConfig.listenPort = 0

        return TunnelConfiguration(name: nil, interface: interfaceConfig, peers: peerConfigs)
    }

    var dnsServers: [IPAddress] {
        let mullvadEndpoint = selectorResult.endpoint
        let dnsSettings = tunnelSettings.dnsSettings

        if dnsSettings.effectiveEnableCustomDNS {
            let dnsServers = dnsSettings.customDNSDomains
                .prefix(DNSSettings.maxAllowedCustomDNSDomains)
            return Array(dnsServers)
        } else {
            if let serverAddress = dnsSettings.blockingOptions.serverAddress {
                return [serverAddress]
            } else {
                return [mullvadEndpoint.ipv4Gateway, mullvadEndpoint.ipv6Gateway]
            }
        }
    }
}
