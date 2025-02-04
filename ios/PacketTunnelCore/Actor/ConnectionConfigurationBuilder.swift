//
//  ConnectionConfigurationBuilder.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-09-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import WireGuardKitTypes

protocol Configuration {
    var name: String { get }
    func make() throws -> ConnectionConfiguration
}

struct ConnectionConfiguration {
    let entryConfiguration: TunnelAdapterConfiguration?
    let exitConfiguration: TunnelAdapterConfiguration
}

struct ConnectionConfigurationBuilder {
    enum ConnectionType {
        case normal
        case ephemeral(EphemeralPeerNegotiationState)
    }

    let type: ConnectionType
    let settings: Settings
    let connectionData: State.ConnectionData

    func make() throws -> ConnectionConfiguration {
        switch type {
        case .normal:
            try NormalConnectionConfiguration(settings: settings, connectionData: connectionData).make()
        case let .ephemeral(ephemeralPeerNegotiationState):
            try EphemeralConnectionConfiguration(
                settings: settings,
                connectionData: connectionData,
                ephemeralPeerNegotiationState: ephemeralPeerNegotiationState
            ).make()
        }
    }
}

private struct NormalConnectionConfiguration: Configuration {
    let settings: Settings
    let connectionData: State.ConnectionData

    var name: String {
        "Normal connection configuration"
    }

    private var activeKey: PrivateKey {
        switch connectionData.keyPolicy {
        case .useCurrent:
            settings.privateKey
        case let .usePrior(priorKey, _):
            priorKey
        }
    }

    func make() throws -> ConnectionConfiguration {
        let entryConfiguration: TunnelAdapterConfiguration? = if connectionData.selectedRelays.entry != nil {
            try ConfigurationBuilder(
                privateKey: activeKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: connectionData.connectedEndpoint,
                allowedIPs: [
                    IPAddressRange(from: "\(connectionData.selectedRelays.exit.endpoint.ipv4Relay.ip)/32")!,
                ],
                pingableGateway: IPv4Address(LocalNetworkIPs.gatewayAddress.rawValue)!
            ).makeConfiguration()
        } else {
            nil
        }
        let exitConfiguration = try ConfigurationBuilder(
            privateKey: activeKey,
            interfaceAddresses: settings.interfaceAddresses,
            dns: settings.dnsServers,
            endpoint: (entryConfiguration != nil)
                ? connectionData.selectedRelays.exit.endpoint
                : connectionData.connectedEndpoint,
            allowedIPs: [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!,
            ],
            pingableGateway: IPv4Address(LocalNetworkIPs.gatewayAddress.rawValue)!
        ).makeConfiguration()

        return ConnectionConfiguration(
            entryConfiguration: entryConfiguration,
            exitConfiguration: exitConfiguration
        )
    }
}

private struct EphemeralConnectionConfiguration: Configuration {
    let settings: Settings
    let connectionData: State.ConnectionData
    let ephemeralPeerNegotiationState: EphemeralPeerNegotiationState

    var name: String {
        "Ephemeral connection configuration"
    }

    func make() throws -> ConnectionConfiguration {
        switch ephemeralPeerNegotiationState {
        case let .single(hop):
            let exitConfiguration = try ConfigurationBuilder(
                privateKey: hop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: connectionData.connectedEndpoint,
                allowedIPs: hop.configuration.allowedIPs,
                preSharedKey: hop.configuration.preSharedKey,
                pingableGateway: IPv4Address(LocalNetworkIPs.gatewayAddress.rawValue)!
            ).makeConfiguration()

            return ConnectionConfiguration(entryConfiguration: nil, exitConfiguration: exitConfiguration)

        case let .multi(firstHop, secondHop):
            let entryConfiguration = try ConfigurationBuilder(
                privateKey: firstHop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: connectionData.connectedEndpoint,
                allowedIPs: firstHop.configuration.allowedIPs,
                preSharedKey: firstHop.configuration.preSharedKey,
                pingableGateway: IPv4Address(LocalNetworkIPs.gatewayAddress.rawValue)!
            ).makeConfiguration()

            let exitConfiguration = try ConfigurationBuilder(
                privateKey: secondHop.configuration.privateKey,
                interfaceAddresses: settings.interfaceAddresses,
                dns: settings.dnsServers,
                endpoint: secondHop.relay.endpoint,
                allowedIPs: secondHop.configuration.allowedIPs,
                preSharedKey: secondHop.configuration.preSharedKey,
                pingableGateway: IPv4Address(LocalNetworkIPs.gatewayAddress.rawValue)!
            ).makeConfiguration()

            return ConnectionConfiguration(entryConfiguration: entryConfiguration, exitConfiguration: exitConfiguration)
        }
    }
}
