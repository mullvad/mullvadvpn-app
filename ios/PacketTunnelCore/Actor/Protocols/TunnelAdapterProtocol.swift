//
//  TunnelAdapterProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
@preconcurrency import WireGuardKitTypes

/// Protocol describing interface for any kind of adapter implementing a VPN tunnel.
public protocol TunnelAdapterProtocol: Sendable {
    /// Start tunnel adapter or update active configuration.
    func start(configuration: TunnelAdapterConfiguration, daita: DaitaConfiguration?) async throws

    /// Start tunnel adapter or update active configuration.
    func startMultihop(
        entryConfiguration: TunnelAdapterConfiguration?,
        exitConfiguration: TunnelAdapterConfiguration,
        daita: DaitaConfiguration?
    ) async throws

    /// Stop tunnel adapter with the given configuration.
    func stop() async throws
}

/// Struct describing tunnel adapter configuration.
public struct TunnelAdapterConfiguration: Sendable {
    public var privateKey: PrivateKey
    public var interfaceAddresses: [IPAddressRange]
    public var dns: [IPAddress]
    public var peer: TunnelPeer?
    public var allowedIPs: [IPAddressRange]
}

/// Struct describing a single peer.
public struct TunnelPeer: Sendable {
    public var endpoint: AnyIPEndpoint
    public var publicKey: PublicKey
    public var preSharedKey: PreSharedKey?
}
