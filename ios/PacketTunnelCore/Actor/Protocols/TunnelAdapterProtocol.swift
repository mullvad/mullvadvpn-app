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

import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/// Protocol describing interface for any kind of adapter implementing a VPN tunnel.
public protocol TunnelAdapterProtocol {
    /// Start tunnel adapter or update active configuration.
    func start(configuration: TunnelAdapterConfiguration) async throws

    /// Stop tunnel adapter with the given configuration.
    func stop() async throws
}

/// Struct describing tunnel adapter configuration.
public struct TunnelAdapterConfiguration {
    public var privateKey: PrivateKey
    public var interfaceAddresses: [IPAddressRange]
    public var dns: [IPAddress]
    public var peer: TunnelPeer?
}

/// Struct describing a single peer.
public struct TunnelPeer {
    public var endpoint: AnyIPEndpoint
    public var publicKey: PublicKey
}
