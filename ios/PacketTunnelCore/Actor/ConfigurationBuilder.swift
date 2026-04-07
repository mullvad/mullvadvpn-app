//
//  ConfigurationBuilder.swift
//  PacketTunnel
//
//  Created by pronebird on 30/08/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

import class WireGuardKitTypes.PreSharedKey
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/// Error returned when there is an endpoint but its public key is invalid.
public struct PublicKeyError: LocalizedError {
    let endpoint: SelectedEndpoint

    public var errorDescription: String? {
        "Public key is invalid, endpoint: \(endpoint)"
    }
}

/// Struct building tunnel adapter configuration.
public struct ConfigurationBuilder {
    var privateKey: WireGuard.PrivateKey
    var interfaceAddresses: [IPAddressRange]
    var dns: SelectedDNSServers?
    var endpoint: SelectedEndpoint?
    var allowedIPs: [IPAddressRange]
    var preSharedKey: WireGuard.PreSharedKey?
    var pingableGateway: IPv4Address

    public init(
        privateKey: WireGuard.PrivateKey,
        interfaceAddresses: [IPAddressRange],
        dns: SelectedDNSServers? = nil,
        endpoint: SelectedEndpoint? = nil,
        allowedIPs: [IPAddressRange],
        preSharedKey: WireGuard.PreSharedKey? = nil,
        pingableGateway: IPv4Address
    ) {
        self.privateKey = privateKey
        self.interfaceAddresses = interfaceAddresses
        self.dns = dns
        self.endpoint = endpoint
        self.allowedIPs = allowedIPs
        self.preSharedKey = preSharedKey
        self.pingableGateway = pingableGateway
    }

    public func makeConfiguration() throws -> TunnelAdapterConfiguration {
        return TunnelAdapterConfiguration(
            privateKey: privateKey,
            interfaceAddresses: interfaceAddresses,
            dns: dnsServers,
            peer: try peer,
            allowedIPs: allowedIPs,
            pingableGateway: pingableGateway
        )
    }

    private var peer: TunnelPeer? {
        get throws {
            guard let endpoint else { return nil }

            guard let publicKey = WireGuard.PublicKey(rawValue: endpoint.publicKey) else {
                throw PublicKeyError(endpoint: endpoint)
            }

            // Socket address is already resolved (IPv4 or IPv6) during relay selection
            return TunnelPeer(
                endpoint: endpoint.socketAddress,
                publicKey: publicKey,
                preSharedKey: preSharedKey
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
