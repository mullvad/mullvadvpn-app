//
//  GotaTunTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
@preconcurrency import NetworkExtension

/// GotaTun tunnel implementation.
/// Unlike WireGuardGo, this implementation does NOT use an external state observer.
/// State transitions are handled internally by the GotaTun actor.
public final class GotaTunTunnelImplementation: TunnelImplementation, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunTunnelImplementation")

    private let _actor = GotaTunActor()
    public var actor: any PacketTunnelActorProtocol { _actor }

    public init() {}

    public func setUp(
        provider: NEPacketTunnelProvider,
        internalQueue: DispatchQueue,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager,
        apiTransportProvider: APITransportProvider
    ) {
        // GotaTun-specific setup will go here when the actor is no longer a stub.
        // The actor will internally manage state transitions, path observation,
        // and system API calls through its own mechanisms.
    }

    public func startTunnel(options: StartOptions) async {
        // NO startObservingActorState() - this is the key architectural difference.
        // The GotaTun actor handles state internally.
        actor.start(options: options)
    }

    public func stopTunnel() async {
        actor.stop()
        await actor.waitUntilDisconnected()
    }

    public func sleep() async {
        actor.onSleep()
    }

    public func wake() {
        actor.onWake()
    }
}
