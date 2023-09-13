//
//  ConfigurationBuilder.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

import protocol Network.IPAddress

/// Struct building tunnel adapter configuration.
struct ConfigurationBuilder {
    var usedKeyPolicy: UsedKeyPolicy
    var deviceData: StoredDeviceData?
    var dns: DNSSettings?
    var endpoint: MullvadEndpoint?

    func makeConfiguration() -> TunnelAdapterConfiguration {
        return TunnelAdapterConfiguration(
            privateKey: privateKey,
            interfaceAddresses: interfaceAddresses,
            dns: dnsServers,
            peer: peer
        )
    }

    private var peer: TunnelPeer? {
        guard let endpoint else { return nil }

        return TunnelPeer(
            endpoint: .ipv4(endpoint.ipv4Relay),
            publicKey: PublicKey(rawValue: endpoint.publicKey)!
        )
    }

    private var interfaceAddresses: [IPAddressRange] {
        guard let deviceData else { return [] }

        return [deviceData.ipv4Address, deviceData.ipv6Address]
    }

    private var privateKey: PrivateKey {
        switch usedKeyPolicy {
        case .useCurrent:
            return deviceData?.wgKeyData.privateKey ?? PrivateKey()

        case let .usePrior(priorKey, _):
            return priorKey
        }
    }

    private var dnsServers: [IPAddress] {
        guard let dns else { return [] }

        if dns.effectiveEnableCustomDNS {
            return Array(dns.customDNSDomains.prefix(DNSSettings.maxAllowedCustomDNSDomains))
        } else {
            if let serverAddress = dns.blockingOptions.serverAddress {
                return [serverAddress]
            } else {
                guard let endpoint else { return [] }

                return [endpoint.ipv4Gateway, endpoint.ipv6Gateway]
            }
        }
    }
}
