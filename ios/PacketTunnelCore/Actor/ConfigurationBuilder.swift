//
//  ConfigurationBuilder.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import protocol Network.IPAddress
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/// Struct building tunnel adapter configuration.
struct ConfigurationBuilder {
    var privateKey: PrivateKey
    var interfaceAddresses: [IPAddressRange]
    var dns: SelectedDNSServers?
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

    private var dnsServers: [IPAddress] {
        guard let dns else { return [] }

        switch dns {
        case let .blocking(dnsAddress):
            return [dnsAddress]
        case let .custom(customDNSAddresses):
            return customDNSAddresses
        case .gateway:
            guard let endpoint else { return [] }
            return [endpoint.ipv4Gateway, endpoint.ipv6Gateway]
        }
    }
}
