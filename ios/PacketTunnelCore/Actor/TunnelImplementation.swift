//
//  TunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
@preconcurrency import NetworkExtension

/// Protocol for tunnel backend implementations.
/// The picker (`PacketTunnelProvider`) delegates all lifecycle events to the active implementation.
public protocol TunnelImplementation: AnyObject {
    /// The underlying actor that handles tunnel state.
    var actor: any PacketTunnelActorProtocol { get }

    /// Called once after init to wire up dependencies.
    /// The `provider` reference is the real `NEPacketTunnelProvider` that iOS instantiated,
    /// needed for system calls like `reasserting` and `setTunnelNetworkSettings`.
    func setUp(
        provider: NEPacketTunnelProvider,
        internalQueue: DispatchQueue,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager,
        apiTransportProvider: APITransportProvider
    )

    /// Called when the tunnel is starting, after initial network settings have been applied.
    func startTunnel(options: StartOptions) async

    /// Called when the tunnel is stopping. Must wait until disconnected before returning.
    func stopTunnel() async

    /// Called when the device is going to sleep.
    func sleep() async

    /// Called when the device wakes up.
    func wake()
}
