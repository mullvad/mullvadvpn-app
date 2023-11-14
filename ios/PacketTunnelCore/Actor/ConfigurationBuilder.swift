//
//  ConfigurationBuilder.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import WireGuardKitTypes

/// Error returned when there is an endpoint but its public key is invalid.
public struct PublicKeyError: LocalizedError {
    let endpoint: MullvadEndpoint

    public var errorDescription: String? {
        "Public key is invalid, endpoint: \(endpoint)"
    }
}

/// Struct building tunnel adapter configuration.
struct ConfigurationBuilder {
    var privateKey: PrivateKey
    var interfaceAddresses: [IPAddressRange]
    var dns: SelectedDNSServers?
    var endpoint: MullvadEndpoint?

    func makeConfiguration() throws -> TunnelAdapterConfiguration {
        return TunnelAdapterConfiguration(
            privateKey: privateKey,
            interfaceAddresses: interfaceAddresses,
            dns: dnsServers,
            peer: try peer
        )
    }

    private var peer: TunnelPeer? {
        get throws {
            guard let endpoint else { return nil }

            guard let publicKey = PublicKey(rawValue: endpoint.publicKey) else {
                throw PublicKeyError(endpoint: endpoint)
            }

            return TunnelPeer(
                endpoint: .ipv4(endpoint.ipv4Relay),
                publicKey: publicKey
            )
        }
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
