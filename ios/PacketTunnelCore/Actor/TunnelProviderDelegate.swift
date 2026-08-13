//
//  TunnelProviderDelegate.swift
//  PacketTunnelCore
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Abstracts the packet tunnel provider capabilities needed to run a tunnel.
///
/// The concrete implementation lives in the `PacketTunnel` target where
/// `NEPacketTunnelProvider` and `NetworkExtension` are available.
public protocol TunnelProviderDelegate: Sendable {
    /// The TUN device file descriptor, or nil if not yet available.
    var tunnelFileDescriptor: Int32? { get }

    /// Apply network settings to the system tunnel interface.
    func applyNetworkSettings(_ settings: TunnelInterfaceSettings) async throws

    /// Toggle the provider's `reasserting` flag to invalidate sockets bound to the old relay.
    func reassertTunnel()
}
